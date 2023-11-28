up.compiler(".map", (el, data) => {
  console.log("Received data:", data);

  const cy = cytoscape({
    container: el,

    elements: { nodes: data },
    layout: {
      //name: "cose",
      name: "preset",
    },
  });
});
