// https://stackoverflow.com/questions/43118692/typescript-filter-out-nulls-from-an-array
export function notNull<TValue>(
    value: TValue | null | undefined
): value is TValue {
    if (value === null || value === undefined) return false;
    const testDummy: TValue = value;
    return true;
}

/**
 * Call the original function, but applying the throttle rules.
 *
 * If the throttled function can be run immediately, this calls it and returns its return
 * value, otherwise, it returns the return value of the last invocation.
 */
export type ThrottledFunction<T extends (...args: any[]) => any> = (
    ...args: Parameters<T>
) => ReturnType<T>;

/**
 * Creates a throttled function that only invokes the provided function (`func`) at most once per within a given number of milliseconds
 * (`limit`)
 */
export function throttle<T extends (...args: any[]) => any>(
    func: T,
    limit: number
): ThrottledFunction<T> {
    let lastResult: ReturnType<T>;
    let lastExecution: number | null = null;
    let scheduled: boolean = false;
    let lastArgs: Parameters<T> | null = null;
    return (...args: Parameters<T>) => {
        const now = performance.now();
        lastArgs = args;
        if (
            lastExecution === null ||
            (!scheduled && now > lastExecution + limit)
        ) {
            lastExecution = now;
            lastResult = func(...lastArgs);
        } else if (!scheduled) {
            scheduled = true;
            const remaining = lastExecution + limit - now;
            setTimeout(function () {
                if (lastArgs === null) throw new Error("unreachable");
                lastExecution = now;
                scheduled = false;
                lastResult = func(...lastArgs);
            }, limit);
        }
        return lastResult;
    };
}

export function shallow_equals<T>(lhs: T, rhs: T) {
    for (let key in lhs) {
        if (lhs[key] != rhs[key]) return false;
    }
    return true;
}
