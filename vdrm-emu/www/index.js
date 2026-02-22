// If you only use `npm` you can simply
// import { Chart } from "wasm-demo" and remove `setup` call from `bootstrap.js`.
class Chart {}

const plotType = document.getElementById("plot-type");
const canvas = document.getElementById("canvas");
const coord = document.getElementById("coord");
const showall = document.getElementById("showall");
const angle = document.getElementById("angle");
const pitch = document.getElementById("pitch");
const yaw = document.getElementById("yaw");
const screen_check = document.getElementById("screen_check");
const control = document.getElementById("3d-control");

const screen_check_2d = document.getElementById("screen_check_2d");
const angle_2d = document.getElementById("angle_2d");
const control_2d = document.getElementById("2d-control");

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
  plotType.addEventListener("change", updatePlot);
  showall.addEventListener("change", updatePlot);
  screen_check.addEventListener("change", updatePlot);
  angle.addEventListener("change", updatePlot);
  angle.addEventListener("input", updatePlot);
  yaw.addEventListener("change", updatePlot);
  yaw.addEventListener("input", updatePlot);
  pitch.addEventListener("change", updatePlot);
  pitch.addEventListener("input", updatePlot);
  window.addEventListener("resize", setupCanvas);
  window.addEventListener("mousemove", onMouseMove);

  angle_2d.addEventListener("change", updatePlot);
  angle_2d.addEventListener("input", updatePlot);
  screen_check_2d.addEventListener("change", updatePlot);
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
  const start = performance.now();
  var enb_screens = [];
  let angle_opt = showall.checked ? null : angle_value;
  for (var i = 0; i < screen_check.children.length; i++) {
    var childElement = screen_check.children[i];
    if (childElement.checked) {
      enb_screens.push(i);
    }
  }
  Chart.plot3d(canvas, angle_opt, pitch_value, yaw_value, enb_screens);
  const end = performance.now();
  coord.innerText = `angle: ${angle_value} Pitch:${pitch_value}, Yaw:${yaw_value} Rendered in ${Math.ceil(end - start)}ms`;
}

function updatePlot2d() {
  var enb_screens = [];
  let angle_value = Number(angle_2d.value);
  for (var i = 0; i < screen_check_2d.children.length; i++) {
    var childElement = screen_check_2d.children[i];
    if (childElement.checked) {
      enb_screens.push(i);
    }
  }
  Chart.plot2d(canvas, angle_value, enb_screens);
}

/** Redraw currently selected plot. */
function updatePlot() {
  const selected = plotType.selectedOptions[0];
  if (selected.value == "3d-plot") {
    control_2d.classList.add("hide");
    control.classList.remove("hide");
    updatePlot3d();
    return;
  }
  control.classList.add("hide");
  control_2d.classList.remove("hide");
  updatePlot2d();
}
