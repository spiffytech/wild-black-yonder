up.compiler(".map", (el, data) => {
  console.log("Received data:", data);

  const cy = cytoscape({
    container: el,

    elements: data,
    layout: {
      //name: "cose",
      name: "preset",
    },

    style: [
      {
        selector: ".waypoint",
        style: {
          "background-color": "#666",
          shape: "rectangle",
        },
      },
      {
        selector: ".waypoint.planet",
        style: {
          "background-color": "#cc0000",
          shape: "diamond",
        },
      },
      {
        selector: ".waypoint.moon",
        style: {
          "background-color": "#cc0000",
          shape: "pentagon",
        },
      },
      {
        selector: ".waypoint.asteroid",
        style: {
          "background-color": "#0000cc",
          shape: "triangle",
        },
      },
      {
        selector: ".ship",
        style: {
          "background-color": "#00cc00",
          shape: "ellipse",
        },
      },
    ],
  });

  cy.$(".waypoint").lock();
  cy.$(".waypoint.moon").unlock();
  cy.layout({
    name: "fcose",
    //name: "cola",
    idealEdgeLength: 1,
  }).run();
});
