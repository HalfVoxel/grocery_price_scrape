import type { ComparePrice } from "./ComparePrice";
import type { ProductType } from "./ProductType";
import type { SoldInUnit } from "./SoldInUnit";

export interface ParsedItem { name: string, url: string, price: number, compare: ComparePrice | null, product_type: ProductType | null, sold_in_unit: SoldInUnit, unit_weight: number | null, }