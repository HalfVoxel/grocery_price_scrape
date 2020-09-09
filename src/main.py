import requests
import os
import progressbar
import inspect
import hashlib
import gzip
import pickle
import json
import time
from datetime import datetime
from typing import List, Optional
from itertools import zip_longest

store = "01143"
store_long = "ica-kvantum-vallentuna-id_01143"

def iter_chunks(iterable, n, fillvalue=None):
    args = [iter(iterable)] * n
    return zip_longest(*args, fillvalue=fillvalue)

def memoize_to_disk():
    def decorator(original_func):
        os.makedirs("cache", exist_ok=True)
        cache = {}
        source = inspect.getsource(original_func)

        def new_func(*params):
            h = hashlib.md5()
            h.update(json.dumps(params, sort_keys=True).encode("utf-8"))
            h.update(original_func.__name__.encode("utf-8"))
            h.update(source.encode("utf-8"))
            hash = h.hexdigest()

            if hash not in cache:
                filename = f"cache/{hash}.pickle"
                if os.path.isfile(filename):
                    with open(filename, "rb") as f:
                        cache[hash] = pickle.load(f)
                        return cache[hash]

                cache[hash] = original_func(*params)
                with open(filename, "wb") as f:
                    pickle.dump(cache[hash], f)

            return cache[hash]

        return new_func

    return decorator


@memoize_to_disk()
def read_categories():
    r = requests.get(f"https://handla.ica.se/api/product-info/v1/store/{store}/category/catalog80002")
    assert r.ok

    return r.json()

@memoize_to_disk()
def list_category(category: str):
    r = requests.get(f"https://handla.ica.se/api/content/v1/collections/facets/customer-type/B2C/store/{store_long}/products?categories={category}&bb=true")
                    #    https://handla.ica.se/api/content/v1/collections/facets/customer-type/B2C/store/01143/products?categories=%22ica-online-catalog-id_catalog80002&bb=true
    assert r.ok

    return r.json()

@memoize_to_disk()
def get_product_data(product_ids: List[int]):
    if len(product_ids) == 0:
        return []

    time.sleep(0.5)
    url = f"https://handla.ica.se/api/content/v1/collection/customer-type/B2C/store/{store_long}/products-data?skus={','.join(product_ids)}"
    r = requests.get(url)
    if not r.ok:
        if len(product_ids) > 1:
            print("Error when accessing product IDs, breaking up into smaller chunks")
            first_half = product_ids[:len(product_ids)//2]
            second_half = product_ids[len(product_ids)//2:]
            d1 = get_product_data(first_half)
            d2 = get_product_data(second_half)
            return [product for h in [d1, d2] for product in h]
        else:
            print("Error when accessing " + str(product_ids[0]) + " skipping")
            print(url)
            print(r.text)
            return []

    assert r.ok
    return r.json()

# @memoize_to_disk()
def get_product_data_chunked(product_ids: List[int]):
    all_products = []
    for chunk in progressbar.progressbar(list(iter_chunks(product_ids, 50)), redirect_stdout=True):
        chunk = [x for x in chunk if x is not None]
        products = get_product_data(chunk)
        for product in products:
            # print(json.dumps(product))
            all_products.append(product)
        
        # print(len(all_products))
    
    return all_products

    
def expect(v: bool):
    if not v:
        print("A a soft assert failed. Something may have failed.")

def main():
    root_category = read_categories()

    all_items = list_category(root_category["seoUrl"])["items"]

    seen_items = set()
    unique_items = []
    for item in all_items:
        if not item["id"] in seen_items:
            assert item["id"] is not None
            seen_items.add(item["id"])
            unique_items.append(item)

    expect(len(all_items) > 10000)

    products = get_product_data_chunked([item["id"] for item in unique_items if item["type"] == "product"])
    
    today = datetime.today()
    dirpath = f"data/{store}/{today.strftime('%Y-%m-%d')}"
    os.makedirs(dirpath)

    with gzip.GzipFile(dirpath + "/products.json.pickle.gzip", 'wb') as file:
        file.write(pickle.dumps(products))

main()
