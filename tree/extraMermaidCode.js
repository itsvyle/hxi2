async function renderSVG(src) {
    const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

    const out = await mermaid.render("mermaid", src);
    const element = document.body;
    element.innerHTML = out.svg;
    await sleep(1);
    const svg = element.querySelector("svg");
    const ch = [...svg.getElementsByClassName("studentnode")];
    const elems = {};
    ch.forEach((e) => {
        let ids = e.id.split("-");
        if (ids.length < 2) return;
        let id = parseInt(ids[1]);
        if (!id) return;
        const b = e.getCTM();
        elems[id] = {
            x: b.e,
            y: b.f,
        };
    });
    return JSON.stringify({
        SVG: out.svg,
        svgWidth: svg.getBBox().width,
        svgHeight: svg.getBBox().height,
        elements: elems,
    });
}
