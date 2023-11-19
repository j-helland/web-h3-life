import './App.css'
import { Component, Show, createSignal } from 'solid-js';
import MapGL, { Viewport, Source, Layer } from 'solid-map-gl';
import * as maplibre from 'maplibre-gl';
import 'maplibre-gl/dist/maplibre-gl.css';
import type { FeatureCollection } from "geojson";

import init, { Universe } from "core";

const App: Component = () => {
  const [coreLoaded, setCoreLoaded] = createSignal(false);
  const [getUniverse, setUniverse] = createSignal(undefined as Universe);
  const [getGeojson, setGeojson] = createSignal({});
  const [viewport, setViewport] = createSignal({
    center: [-105.27080358180076, 40.01409069474372],
    zoom: 0,
  } as Viewport);

  const numInitCells = 64;
  const h3Resolution = 2;

  init().then(() => {
    setCoreLoaded(true);
    setGeojson({
      type: "geojson",
      // dummy initialization
      data: { type: "Feature", geometry: { type: "Point", coordinates: [] } },
    });
    setUniverse(Universe.new(numInitCells, h3Resolution));
    // After a single tick, most cells die and a few clusters remain. Start displaying after this pruning.
    getUniverse().tick(); 
  });

  setInterval(() => {
      // Guard on WASM module initialization.
      if (!coreLoaded()) {
        return;
      }

      const universe: Universe = getUniverse();
      if (universe == null) {
        return;
      }

      const geojson_raw = universe.render();
      const data = JSON.parse(geojson_raw) as FeatureCollection;
      setGeojson({
        type: "geojson",
        data: data,
      });
      universe.tick();
  }, 690); 

  return (
    <Show when={coreLoaded()}>
      <MapGL
        mapLib={maplibre}
        options={{ style: 'assets/style.json' }}
        viewport={viewport()}
        onViewportChange={(evt: Viewport) => setViewport(evt)}
      >
        <Source
          source={getGeojson()}
        >
          <Layer
            style={{
              type: "fill",
              paint: {
                "fill-color": "#AD66CC",
                "fill-opacity": 0.5,
              },
            }}
          />
          <Layer
            style={{
              type: "line",
              paint: {
                "line-color": "#000",
                "line-width": 1,
              }
            }}
          />
        </Source>
      </MapGL>
    </Show>
  );
}

export default App
