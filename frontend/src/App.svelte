<script lang="ts">
    import { onMount } from "svelte";
    import {
        CategoryScale,
        Chart,
        ChartDataset,
        Decimation,
        Filler,
        Legend,
        LinearScale,
        LineController,
        LineElement,
        LogarithmicScale,
        PointElement,
        ScatterController,
        TimeScale,
        TimeSeriesScale,
        Title,
        Tooltip,
    } from "chart.js";
    import { throttle } from "./utilities";
    import type { Item } from "./bindings/Item";
    import "chartjs-adapter-luxon";
    import {
        ExternalLinkIcon,
        Link2Icon,
        LinkIcon,
    } from "svelte-feather-icons";

    Chart.register(
        LineElement,
        PointElement,
        LineController,
        ScatterController,
        CategoryScale,
        LinearScale,
        LogarithmicScale,
        TimeScale,
        TimeSeriesScale,
        Decimation,
        Filler,
        Legend,
        Title,
        Tooltip
    );

    const colors = [
        // "#a6cee3",
        // "#1f78b4",
        // "#b2df8a",
        // "#33a02c",
        // "#fb9a99",
        // "#e31a1c",
        // "#fdbf6f",
        // "#ff7f00",
        // "#cab2d6",
        // "#6a3d9a",
        // "#ffff99",
        // "#b15928",

        "#003f5c",
        "#2f4b7c",
        "#665191",
        "#a05195",
        "#d45087",
        "#f95d6a",
        "#ff7c43",
        "#ffa600",
    ];

    let canvas: HTMLCanvasElement;
    let search: string = "";
    let searchData: Item[] | null = null;

    const trottledSearch = throttle((query) => {
        if (search.length > 0) {
            fetch("/search", {
                method: "POST",
                body: JSON.stringify({
                    name: query,
                }),
            })
                .then((r) => r.json())
                .then((data) => {
                    searchData = data as Item[];
                });
        } else {
            searchData = null;
        }
    }, 500);

    let chart: Chart<"line"> | null = null;
    let included: Set<string> = new Set();
    const item2dataset: Map<string, ChartDataset<"line">> = new Map();

    $: {
        trottledSearch(search);
    }

    $: {
        if (searchData !== null && chart !== null) {
            const dates = searchData.map((x) =>
                x.price_data.map((d) => d.date)
            );
            if (dates.length !== 0) {
                for (const item of searchData) {
                    item.price_data.sort(
                        (a, b) =>
                            new Date(a.date).getTime() -
                            new Date(b.date).getTime()
                    );
                }

                let mn = new Date(dates[0][0]);
                let mx = new Date(dates[0][0]);
                for (let ds of dates) {
                    for (let d of ds) {
                        const date = new Date(d);
                        mn = new Date(Math.min(mn.getTime(), date.getTime()));
                        mx = new Date(Math.max(mx.getTime(), date.getTime()));
                    }
                }

                let dateRange = [];
                while (mn < mx) {
                    dateRange.push(new Date(mn));
                    mn.setDate(mn.getDate() + 1);
                }

                const stores = searchData;

                const datasets = [];
                let counter = -1;
                for (const item of searchData) {
                    counter += 1;
                    if (!included.has(item.name)) continue;

                    let dataset = item2dataset.get(item.name) || {
                        label: item.name,
                        data: [],
                        borderWidth: 3,
                        tension: 0.5,
                        borderColor: colors[counter % colors.length],
                        stepped: "middle",
                    };

                    let data = item.price_data
                        .filter(
                            (x) => x.store_id === item.price_data[0].store_id
                        )
                        .map((x) => {
                            return {
                                x: x.date as any as number,
                                y: x.data.compare?.price || 0,
                            };
                        });

                    dataset.data = data;
                    item2dataset.set(item.name, dataset);
                    datasets.push(dataset);
                }

                chart.data.labels = []; //dateRange;
                chart.data.datasets = datasets;
                // console.log(datasets);
                chart.update();
            }
        }
    }

    onMount(() => {
        const ctx = canvas.getContext("2d");
        if (ctx === null) throw new Error("Canvas not supported");
        chart = new Chart(ctx, {
            type: "line",

            data: {
                // labels: ["Red", "Blue", "Yellow", "Green", "Purple", "Orange"],
                labels: ["2021-01-01", "2021-01-02"],
                datasets: [
                    {
                        label: "# of Votes",
                        data: [0, 2],
                        borderWidth: 4,
                        tension: 0.5,
                        stepped: "middle",
                        spanGaps: false,
                    },
                ],
            },
            options: {
                responsive: true,
                scales: {
                    x: {
                        type: "time",
                        time: {
                            unit: "month",
                            displayFormats: {
                                quarter: "YYYY MMM",
                            },
                        },
                    },
                    y: {
                        beginAtZero: true,
                    },
                },
                interaction: {
                    mode: "nearest",
                    intersect: false,
                },
            },
        });
    });
</script>

<main>
    <div class="sidebar">
        <input class="search" type="text" bind:value={search} autofocus />
        {#if searchData !== null}
            <ol>
                {#each searchData as item, i}
                    <li
                        style={included.has(item.name)
                            ? `background-color: ${colors[i % colors.length]}`
                            : ""}
                        class:included={included.has(item.name)}
                        on:click={(e) => {
                            if (included.has(item.name)) {
                                included.delete(item.name);
                            } else {
                                included.add(item.name);
                            }
                            included = included;
                        }}
                    >
                        <!-- <input
                            type="checkbox"
                            checked={included.has(item.name)}
                            id={`include-${item.name}`}
                            on:change={(e) => {
                                if (e.currentTarget.checked) {
                                    included.add(item.name);
                                    included = included;
                                } else {
                                    included.delete(item.name);
                                    included = included;
                                }
                            }}
                        /> -->
                        <label for={`include-${item.name}`}>{item.name}</label>
                        <a on:click={(e) => e.stopPropagation()} href={item.url}
                            ><ExternalLinkIcon /></a
                        >
                    </li>
                {/each}
            </ol>
        {/if}
    </div>
    <div class="canvas">
        <canvas bind:this={canvas} />
    </div>
</main>
