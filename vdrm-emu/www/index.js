// If you only use `npm` you can simply
// import { Chart } from "wasm-demo" and remove `setup` call from `bootstrap.js`.
class Chart {}

const canvas = document.getElementById("canvas");
const coord = document.getElementById("coord");
const showall = document.getElementById("showall");
const angle = document.getElementById("angle");
const pitch = document.getElementById("pitch");
const yaw = document.getElementById("yaw");
const min_angle = document.getElementById("min_angle");
const max_angle = document.getElementById("max_angle");

let chart = null;

/** Main entry point */
export function main() {
  setupUI();
  setupCanvas();
}

/** This function is used in `bootstrap.js` to setup imports. */
export function setup(WasmChart) {
  Chart = WasmChart;
}

/** Add event listeners. */
function setupUI() {
  showall.addEventListener("change", updatePlot);
  angle.addEventListener("change", updatePlot);
  angle.addEventListener("input", updatePlot);
  yaw.addEventListener("change", updatePlot);
  yaw.addEventListener("input", updatePlot);
  pitch.addEventListener("change", updatePlot);
  pitch.addEventListener("input", updatePlot);
  min_angle.addEventListener("change", updatePlot);
  min_angle.addEventListener("input", updatePlot);
  max_angle.addEventListener("change", updatePlot);
  max_angle.addEventListener("input", updatePlot);
  window.addEventListener("resize", setupCanvas);
  window.addEventListener("mousemove", onMouseMove);
}

/** Setup canvas to properly handle high DPI and redraw current plot. */
function setupCanvas() {
  const dpr = window.devicePixelRatio || 1.0;
  const aspectRatio = canvas.width / canvas.height;
  const size = canvas.parentNode.offsetWidth * 0.8;
  canvas.style.width = size + "px";
  canvas.style.height = size / aspectRatio + "px";
  canvas.width = size;
  canvas.height = size / aspectRatio;
  updatePlot();
}

/** Update displayed coordinates. */
function onMouseMove(event) {
  if (chart) {
    var text = "Mouse pointer is out of range";

    if (event.target == canvas) {
      let actualRect = canvas.getBoundingClientRect();
      let logicX = (event.offsetX * canvas.width) / actualRect.width;
      let logicY = (event.offsetY * canvas.height) / actualRect.height;
      const point = chart.coord(logicX, logicY);
      text = point ? `(${point.x.toFixed(3)}, ${point.y.toFixed(3)})` : text;
    }
    coord.innerText = text;
  }
}

function updatePlot3d() {
  angle.disabled = showall.checked;
  let angle_value = Number(angle.value);
  let yaw_value = Number(yaw.value) / 100.0;
  let pitch_value = Number(pitch.value) / 100.0;
  let min_angle_value = Number(min_angle.value);
  let max_angle_value = Number(max_angle.value);
  const start = performance.now();
  let angle_opt = showall.checked ? null : angle_value;
  Chart.plot3d(
    canvas,
    angle_opt,
    pitch_value,
    yaw_value,
    min_angle_value,
    max_angle_value,
  );
  const end = performance.now();
  coord.innerText = `angle: ${angle_value} in (${min_angle_value}..${max_angle_value}) Pitch:${pitch_value}, Yaw:${yaw_value} Rendered in ${Math.ceil(end - start)}ms`;
}

/** Redraw currently selected plot. */
function updatePlot() {
  updatePlot3d();
}
