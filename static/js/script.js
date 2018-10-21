class TerrainPreview {
  constructor(element) {
    this.is_running = false;

    this.scene = new THREE.Scene();

    const width = element.offsetWidth;
    const height = Math.round(width / 16 * 9);

    this.renderer = new THREE.WebGLRenderer();
    this.renderer.setSize(width, height);
    this.renderer.setPixelRatio(window.devicePixelRatio);
    this.renderer.setClearColor(0xffffff, 1);

    element.appendChild(this.renderer.domElement);

    this.camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
    this.camera.position.x = 0;
    this.camera.position.y = 70;
    this.camera.position.z = 100;
    this.camera.lookAt(0, 0, 0);

    this.rotation = 0;
  }

  setRiver(line) {
    let geometry = new THREE.Geometry();
    let material = new THREE.LineBasicMaterial( {
      color: 0x0000ff,
      linewidth: 2,
    });

    for (let i = 0; i < line.length; i += 3) {
      geometry.vertices.push(
        new THREE.Vector3(
          line[i] * 0.002,
          line[i + 2] * 0.02,
          line[i + 1] * 0.002
        )
      );
    }

    let mesh = new THREE.LineSegments(geometry, material);

    this.scene.remove(this.river_mesh);
    this.scene.add(mesh);

    this.river_mesh = mesh;
  }

  setContour(line) {
    let geometry = new THREE.Geometry();
    let material = new THREE.LineBasicMaterial( {
      color: 0x222222,
      linewidth: 2,
    });

    for (let i = 0; i < line.length; i += 2) {
      geometry.vertices.push(
        new THREE.Vector3(
          line[i] * 0.002,
          0,
          line[i + 1] * 0.002
        )
      );
    }

    let mesh = new THREE.LineLoop(geometry, material);

    this.scene.remove(this.contour_mesh);
    this.scene.add(mesh);

    this.contour_mesh = mesh;
  }

  render(once) {
    if (this.is_running && !once) {
      requestAnimationFrame(() => this.render());

      this.rotation += 0.002;
    }

    this.river_mesh.rotation.y = this.rotation;
    this.contour_mesh.rotation.y = this.rotation;

    this.renderer.render(this.scene, this.camera);
  }

  startRender() {
    this.is_running = true;
    this.render();
  }

  stopRender() {
    this.is_running = false;
  }
}

class SlopeMapEditor {
  constructor(element, size, max, default_slope_map) {
    this.element = element;
    this.size = size;
    this.max = max;
    this.map = new Array(size * size).fill(0, 0, size * size);

    if (default_slope_map) {
      this.map = default_slope_map;
    }

    this.canvas = document.createElement('canvas');
    this.button = document.createElement('button');
    this.strength = document.createElement('input');
    this.erase = document.createElement('input');

    this.canvas.width = 400;
    this.canvas.height = 400;
    this.scale = this.canvas.width / this.size;
    this.context = this.canvas.getContext('2d');

    this.button.textContent = 'Reset';

    this.strength.type = 'range'
    this.strength.min = 0.005;
    this.strength.max = 0.02;
    this.strength.step = 0.0001;
    this.strength.value = 0.017;

    this.erase.type = 'checkbox';

    element.appendChild(this.canvas);
    element.appendChild(this.button);
    element.appendChild(this.strength);
    element.appendChild(this.erase);

    this.render();

    this.interval = null;
    this.paint_x = 0;
    this.paint_y = 0;

    this.button.addEventListener('click', () => {
      this.map = new Array(size * size).fill(0, 0, size * size);
      this.render();
    });

    this.canvas.addEventListener('mousedown', (e) => {
      e.preventDefault();

      this.interval = setInterval(() => {
        const strength = parseFloat(this.strength.value);
        const erase = this.erase.checked;

        this.paint(this.paint_x, this.paint_y, 4, strength * (erase ? -1 : 1));
        this.render();
      }, 50);
    });

    this.canvas.addEventListener('mousemove', (e) => {
      this.paint_x = Math.floor(e.offsetX / this.scale);
      this.paint_y = Math.floor(e.offsetY / this.scale);
    });

    window.addEventListener('mouseup', (e) => {
      if (this.interval !== null) {
        clearInterval(this.interval);
        this.interval = null;
      }
    });
  }

  paint(x, y, size, strength) {
    const size_square = size * size;
    for (let i = 0; i < this.map.length; i++) {
      const px = i % this.size;
      const py = Math.floor(i / this.size);

      const d = Math.pow(x - px, 2) + Math.pow(y - py, 2)

      let m = 1;
      if (d > 1) {
        if (d < size_square) {
          m = 1 / d;
        }
        else {
          m = 0;
        }
      }

      this.map[i] = Math.max(Math.min(this.map[i] + strength * m, this.max), 0);
    }
  }

  render() {
    const scale = this.scale;

    this.map.forEach((value, index) => {
      const y = Math.floor(index / this.size);
      const x = index % this.size;
      const v = Math.floor(value * 256) / this.max;

      this.context.fillStyle = 'rgb(' + [v, v, v].join(',') + ')';
      this.context.fillRect(x * scale, y * scale, scale, scale);
    });
  }
}

class ContourEditor {
  constructor(element, max_size, contour) {
    this.element = element;
    this.max_size = max_size;
    this.contour = contour;

    this.canvas = document.createElement('canvas');
    this.canvas.width = 400;
    this.canvas.height = 400;
    this.scale = this.canvas.width / this.max_size / 1.1;
    this.context = this.canvas.getContext('2d');

    this.element.appendChild(this.canvas);

    this.render();

    this.is_dragging = false;
    this.node_selected = null;

    this.canvas.addEventListener('mousemove', (e) => {
      const offset = this.canvas.width / 2;
      const x = (e.offsetX - offset) / this.scale;
      const y = (e.offsetY - offset) / this.scale;

      const active_distance = Math.pow(20 / this.scale, 2);

      let is_dirty = false;

      if (this.is_dragging) {
        this.contour[this.node_selected] = x;
        this.contour[this.node_selected + 1] = y;
        is_dirty = true;
      }
      else {
        const last_node_selected = this.node_selected;
        this.node_selected = null;

        for (let i = 0; i < this.contour.length; i += 2) {
          const node_x = this.contour[i];
          const node_y = this.contour[i + 1];

          const d = Math.pow(node_x - x, 2) + Math.pow(node_y - y, 2);
          if (d < active_distance) {
            this.node_selected = i;
            break;
          }
        }

        is_dirty = (is_dirty || last_node_selected !== this.node_selected);
      }

      if (is_dirty) {
        this.render();
      }
    });

    this.canvas.addEventListener('mousedown', () => {
      if (this.node_selected !== null) {
        this.is_dragging = true;
      }
    });

    window.addEventListener('mouseup', () => {
      this.is_dragging = false;
    });
  }

  render() {
    const scale = this.scale;
    const offset = this.canvas.width / 2;

    this.context.clearRect(0, 0, this.canvas.width, this.canvas.height);

    this.context.strokeStyle = '#111';
    this.context.fillStyle = '#181';

    this.context.beginPath();
    this.context.moveTo(
      Math.floor(this.contour[0] * scale + offset),
      Math.floor(this.contour[1] * scale + offset)
    );

    for (let i = 2; i < this.contour.length; i += 2) {
      this.context.lineTo(
        Math.floor(this.contour[i] * scale + offset),
        Math.floor(this.contour[i + 1] * scale + offset)
      );
    }

    this.context.closePath();
    this.context.stroke();

    for (let i = 0; i < this.contour.length; i += 2) {
      if (i === this.node_selected) {
        this.context.fillStyle = '#881';
      }

      this.context.fillRect(
        Math.floor(this.contour[i] * scale + offset) - 3,
        Math.floor(this.contour[i + 1] * scale + offset) - 3,
        6, 6
      );

      if (i === this.node_selected) {
        this.context.fillStyle = '#181';
      }
    }
  }
}

const default_slope_map = [0.012,0.021,0.019,0.027,0.027,0.029,0.023,0.018,0.019,0.02,0.029,0.018,0.012,0.012,0.011,0.013,0.019,0.025,0.032,0.036,0.041,0.045,0.039,0.016,0.007,0.041,0.033,0.03,0.022,0.028,0.033,0.034,0.042,0.036,0.045,0.037,0.036,0.022,0.03,0.03,0.03,0.031,0.042,0.047,0.05,0.053,0.058,0.047,0.029,0.01,0.057,0.054,0.029,0.024,0.02,0.031,0.042,0.049,0.05,0.05,0.052,0.041,0.046,0.046,0.046,0.04,0.037,0.04,0.058,0.07,0.061,0.058,0.048,0.033,0.014,0.105,0.073,0.04,0.029,0.032,0.056,0.052,0.058,0.071,0.069,0.056,0.054,0.048,0.052,0.042,0.036,0.036,0.044,0.067,0.078,0.084,0.068,0.053,0.033,0.019,0.106,0.095,0.045,0.039,0.039,0.049,0.056,0.068,0.081,0.08,0.065,0.051,0.05,0.044,0.044,0.043,0.044,0.051,0.059,0.076,0.076,0.063,0.051,0.038,0.022,0.14,0.07,0.039,0.031,0.088,0.1,0.045,0.053,0.069,0.078,0.06,0.059,0.062,0.06,0.056,0.053,0.044,0.047,0.047,0.051,0.069,0.067,0.048,0.038,0.022,0.092,0.056,0.025,0.045,0.062,0.107,0.065,0.039,0.053,0.061,0.1,0.112,0.078,0.069,0.057,0.047,0.041,0.057,0.058,0.081,0.075,0.065,0.053,0.038,0.023,0.059,0.03,0.094,0.08,0.113,0.141,0.192,0.038,0.039,0.096,0.195,0.186,0.12,0.064,0.053,0.046,0.067,0.065,0.095,0.09,0.094,0.067,0.046,0.036,0.022,0.019,0.114,0.066,0.055,0.119,0.172,0.153,0.125,0.075,0.094,0.164,0.191,0.208,0.133,0.054,0.081,0.071,0.102,0.096,0.101,0.089,0.062,0.051,0.035,0.021,0.042,0.11,0.05,0.031,0.072,0.112,0.141,0.118,0.108,0.121,0.132,0.132,0.134,0.164,0.1,0.069,0.09,0.099,0.126,0.133,0.091,0.067,0.052,0.033,0.02,0.045,0.184,0.062,0.012,0.058,0.09,0.11,0.141,0.166,0.144,0.139,0.071,0.118,0.141,0.11,0.082,0.089,0.112,0.158,0.186,0.126,0.069,0.045,0.027,0.019,0.053,0.199,0.153,0.089,0.108,0.151,0.136,0.17,0.171,0.181,0.131,0.112,0.116,0.181,0.159,0.123,0.076,0.084,0.165,0.18,0.149,0.082,0.053,0.031,0.016,0.032,0.17,0.207,0.188,0.178,0.167,0.18,0.175,0.202,0.174,0.176,0.154,0.161,0.178,0.183,0.128,0.093,0.087,0.114,0.155,0.133,0.085,0.066,0.032,0.019,0.036,0.043,0.162,0.165,0.192,0.206,0.214,0.225,0.192,0.18,0.181,0.159,0.107,0.175,0.179,0.154,0.111,0.09,0.113,0.12,0.117,0.097,0.067,0.037,0.019,0.038,0.047,0.037,0.032,0.173,0.232,0.244,0.222,0.194,0.177,0.197,0.121,0.136,0.11,0.192,0.135,0.105,0.114,0.102,0.116,0.11,0.093,0.057,0.044,0.016,0.038,0.082,0.078,0.194,0.226,0.249,0.234,0.231,0.217,0.21,0.215,0.181,0.144,0.163,0.138,0.115,0.115,0.132,0.13,0.113,0.092,0.073,0.053,0.032,0.016,0.044,0.103,0.209,0.236,0.236,0.247,0.254,0.246,0.255,0.262,0.252,0.213,0.201,0.186,0.136,0.117,0.14,0.159,0.16,0.13,0.09,0.052,0.044,0.052,0.028,0.021,0.113,0.252,0.24,0.256,0.258,0.269,0.278,0.282,0.279,0.227,0.215,0.2,0.176,0.174,0.149,0.165,0.179,0.165,0.119,0.084,0.045,0.065,0.065,0.053,0.009,0.12,0.258,0.251,0.27,0.28,0.288,0.291,0.266,0.235,0.223,0.183,0.186,0.192,0.156,0.182,0.158,0.159,0.118,0.097,0.057,0.047,0.065,0.091,0.052,0.011,0.103,0.243,0.277,0.286,0.29,0.296,0.29,0.217,0.18,0.164,0.148,0.161,0.158,0.16,0.12,0.137,0.093,0.077,0.053,0.047,0.048,0.08,0.071,0.055,0.026,0.071,0.192,0.294,0.297,0.299,0.3,0.169,0.122,0.118,0.1,0.108,0.127,0.113,0.095,0.085,0.064,0.058,0.05,0.047,0.044,0.067,0.057,0.065,0.031,0.035,0.056,0.115,0.192,0.229,0.211,0.153,0.097,0.079,0.068,0.078,0.083,0.078,0.065,0.058,0.046,0.045,0.04,0.053,0.041,0.063,0.056,0.054,0.027,0.015,0.078,0.066,0.07,0.091,0.094,0.087,0.079,0.071,0.051,0.05,0.069,0.08,0.07,0.053,0.051,0.051,0.039,0.043,0.042,0.055,0.054,0.057,0.032,0.016,0.008,0.076,0.097,0.067,0.074,0.064,0.054,0.059,0.056,0.078,0.096,0.07,0.068,0.053,0.042,0.054,0.047,0.046,0.039,0.043,0.042,0.044,0.032,0.015,0.008,0.003,0.076,0.063,0.077,0.056,0.042,0.038,0.041,0.063,0.105,0.098,0.095,0.047,0.04,0.028,0.03,0.035,0.028,0.026,0.025,0.024,0.021,0.014,0.007,0.004,0.001];

// const default_terrain_contour = [
//   0,0, 0.1,0, 0.2,0, 0.3,0, 0.4,0, 0.5,0, 0.6,0, 0.7,0, 0.8,0, 0.9,0,
//   1,0, 1,0.1, 1,0.2, 1,0.3, 1,0.4, 1,0.5, 1,0.6, 1,0.7, 1,0.8, 1,0.9,
//   1,1, 0.9,1, 0.8,1, 0.7,1, 0.6,1, 0.5,1, 0.4,1, 0.3,1, 0.2,1, 0.1,1,
//   0,1, 0,0.9, 0,0.8, 0,0.7, 0,0.6, 0,0.5, 0,0.4, 0,0.3, 0,0.2, 0,0.1
// ].map((x) => x * 1e5 - 5e4);

// const default_terrain_contour = [
//   0,0, 0.2,0, 0.4,0, 0.6,0, 0.8,0,
//   1,0, 1,0.2, 1,0.4, 1,0.6, 1,0.8,
//   1,1, 0.8,1, 0.6,1, 0.4,1, 0.2,1,
//   0,1, 0,0.8, 0,0.6, 0,0.4, 0,0.2
// ].map((x) => x * 1e5 - 5e4);

const default_terrain_contour = [-24475, -13200, -15675, -22825, -1375, -26675, 7975, -26675, 17875, -21450, 28325, -22275, 37675, -15400, 40975, -5500, 35750, 4675, 36300, 16500, 45100, 25575, 42075, 34650, 28600, 41250, 15125, 37125, 2750, 27500, -11275, 20075, -26950, 21175, -37675, 15400, -42075, 3575, -36850, -6875];

(async function() {

  const terrain_preview_element = document.getElementById('terrain-preview');
  const terrain_stats_element = document.getElementById('terrain-stats');
  const river_slope_map_element = document.getElementById('editor-river-slope-map');
  const terrain_contour_element = document.getElementById('editor-terrain-contour');

  const terrain_preview = new TerrainPreview(terrain_preview_element);
  const river_slope_map = new SlopeMapEditor(river_slope_map_element, 25, 0.3, default_slope_map);
  const terrain_contour = new ContourEditor(terrain_contour_element, 1e5, default_terrain_contour);

  const terrain = await Rust.terrain_wasm;

  const setting_river_growth = document.getElementById('setting-river-growth');
  const setting_river_symmetric = document.getElementById('setting-river-symmetric');
  const setting_river_asymmetric = document.getElementById('setting-river-asymmetric');

  function generate_river() {
    let river_growth = parseFloat(setting_river_growth.value);
    let river_symmetric = parseFloat(setting_river_symmetric.value);
    let river_asymmetric = parseFloat(setting_river_asymmetric.value);

    const normal = river_growth + river_symmetric + river_asymmetric;
    river_growth /= normal;
    river_symmetric /= normal;
    river_asymmetric /= normal;

    const timer_start = window.performance.now();

    const river = terrain.generate_river(
      river_growth, river_symmetric, river_asymmetric,
      river_slope_map.map,
      terrain_contour.contour
    );

    const river_generation_timer = window.performance.now() - timer_start;

    const highest_point = river.filter((x, i) => i % 3 === 2).reduce((a, x) => Math.max(a, x));

    const stats = [
      `generate_river: ${Math.round(river_generation_timer)}ms`,
      `river_edges: ${river.length / 6}`,
      `highest_point: ${Math.round(highest_point)}m`,
    ].join('\n');

    terrain_stats_element.textContent = stats;

    terrain_preview.setRiver(river);
    terrain_preview.setContour(terrain_contour.contour);

    terrain_preview.render(true);
  }

  document.getElementById('action-regenerate').addEventListener('click', () => {
    generate_river();
  });

  document.getElementById('action-start-stop').addEventListener('click', () => {
    if (terrain_preview.is_running) {
      terrain_preview.stopRender();
    }
    else {
      terrain_preview.startRender();
    }
  });

  generate_river();

  terrain_preview.startRender();

})();
