(() => {
    const shared = { payload: 23 };
    const graph = { first: shared, second: shared };
    graph.self = graph;
    graph.map = new Map([[graph, shared]]);
    graph.set = new Set([graph, shared, graph.map]);
    const date = new Date(9876);
    graph.dates = [date, date, new Date(NaN)];
    const copy = structuredClone(graph);
    return copy !== graph && copy.self === copy
        && copy.first === copy.second && copy.first !== shared
        && copy.first.payload === 23 && copy.map.get(copy) === copy.first
        && copy.set.has(copy) && copy.set.has(copy.first) && copy.set.has(copy.map)
        && copy.dates[0] === copy.dates[1] && copy.dates[0] !== date
        && copy.dates[0].getTime() === 9876 && Number.isNaN(copy.dates[2].getTime());
})()
