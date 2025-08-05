import { invoke } from "@tauri-apps/api/tauri";

let greetInputEl;
let greetMsgEl;

// System info elements
let systemInfoBtn;
let cpuTempBtn;
let systemOutput;

// GPIO elements
let gpioPinInput;
let gpioOnBtn;
let gpioOffBtn;
let gpioOutput;

// Sensor elements
let sensorNameInput;
let sensorValueInput;
let addSensorBtn;
let refreshSensorsBtn;
let sensorsGrid;

async function greet() {
  greetMsgEl.textContent = await invoke("greet", { name: greetInputEl.value });
}

async function getSystemInfo() {
  try {
    const info = await invoke("get_system_info");
    systemOutput.innerHTML = `<pre>${info}</pre>`;
  } catch (error) {
    systemOutput.innerHTML = `<p style="color: #ff6b6b;">Error: ${error}</p>`;
  }
}

async function getCpuTemperature() {
  try {
    const temp = await invoke("read_cpu_temperature");
    systemOutput.innerHTML = `<p>CPU Temperature: <span style="color: #4CAF50; font-weight: bold;">${temp.toFixed(1)}°C</span></p>`;
  } catch (error) {
    systemOutput.innerHTML = `<p style="color: #ff6b6b;">Error: ${error}</p>`;
  }
}

async function toggleGpio(state) {
  const pin = parseInt(gpioPinInput.value);
  if (isNaN(pin) || pin < 1 || pin > 40) {
    gpioOutput.innerHTML = `<p style="color: #ff6b6b;">Please enter a valid GPIO pin (1-40)</p>`;
    return;
  }

  try {
    const result = await invoke("toggle_gpio_pin", { pin, state });
    gpioOutput.innerHTML = `<p style="color: #4CAF50;">${result}</p>`;
  } catch (error) {
    gpioOutput.innerHTML = `<p style="color: #ff6b6b;">Error: ${error}</p>`;
  }
}

async function addSensor() {
  const name = sensorNameInput.value.trim();
  const value = parseFloat(sensorValueInput.value);

  if (!name || isNaN(value)) {
    alert("Please enter both sensor name and value");
    return;
  }

  try {
    await invoke("update_sensor", { name, value });
    sensorNameInput.value = "";
    sensorValueInput.value = "";
    refreshSensors();
  } catch (error) {
    alert(`Error: ${error}`);
  }
}

async function refreshSensors() {
  try {
    const sensors = await invoke("get_sensor_data");
    sensorsGrid.innerHTML = "";
    
    sensors.forEach(sensor => {
      const sensorCard = document.createElement("div");
      sensorCard.className = "sensor-card";
      sensorCard.innerHTML = `
        <h4>${sensor.name}</h4>
        <div class="sensor-value">${sensor.value}</div>
        <small>Updated: ${new Date(sensor.timestamp * 1000).toLocaleTimeString()}</small>
      `;
      sensorsGrid.appendChild(sensorCard);
    });
  } catch (error) {
    sensorsGrid.innerHTML = `<p style="color: #ff6b6b;">Error loading sensors: ${error}</p>`;
  }
}

window.addEventListener("DOMContentLoaded", () => {
  greetInputEl = document.querySelector("#greet-input");
  greetMsgEl = document.querySelector("#greet-msg");
  document.querySelector("#greet-btn").addEventListener("click", greet);

  // System info
  systemInfoBtn = document.querySelector("#system-info-btn");
  cpuTempBtn = document.querySelector("#cpu-temp-btn");
  systemOutput = document.querySelector("#system-output");
  
  systemInfoBtn.addEventListener("click", getSystemInfo);
  cpuTempBtn.addEventListener("click", getCpuTemperature);

  // GPIO
  gpioPinInput = document.querySelector("#gpio-pin");
  gpioOnBtn = document.querySelector("#gpio-on-btn");
  gpioOffBtn = document.querySelector("#gpio-off-btn");
  gpioOutput = document.querySelector("#gpio-output");

  gpioOnBtn.addEventListener("click", () => toggleGpio(true));
  gpioOffBtn.addEventListener("click", () => toggleGpio(false));

  // Sensors
  sensorNameInput = document.querySelector("#sensor-name");
  sensorValueInput = document.querySelector("#sensor-value");
  addSensorBtn = document.querySelector("#add-sensor-btn");
  refreshSensorsBtn = document.querySelector("#refresh-sensors-btn");
  sensorsGrid = document.querySelector("#sensors-grid");

  addSensorBtn.addEventListener("click", addSensor);
  refreshSensorsBtn.addEventListener("click", refreshSensors);

  // Initial sensor load
  refreshSensors();
});
