// Détection OS pour pré-sélectionner l'onglet
function detectOS() {
  const ua = navigator.userAgent.toLowerCase();
  if (ua.includes("win")) return "windows";
  if (ua.includes("mac")) return "macos";
  if (ua.includes("linux")) return "linux";
  return "linux";
}

function switchTab(os) {
  document.querySelectorAll(".install-tab").forEach((t) => {
    t.classList.toggle("active", t.dataset.os === os);
  });
  ["linux", "macos", "windows"].forEach((id) => {
    const el = document.getElementById("install-" + id);
    if (el) el.classList.toggle("hidden", id !== os);
  });
}

function copy(btn) {
  const code = btn.parentElement.querySelector("code").innerText;
  navigator.clipboard.writeText(code).then(() => {
    const old = btn.innerText;
    btn.innerText = "Copié !";
    btn.style.color = "#00f2ff";
    setTimeout(() => {
      btn.innerText = old;
      btn.style.color = "";
    }, 1500);
  });
}

// Init onglet selon OS
document.addEventListener("DOMContentLoaded", () => {
  switchTab(detectOS());
});

// je relie les liens de téléchargement aux fichiers de la dernière
// release GitHub : les href codés en dur (repli sur la version
// courante) sont remplacés par ceux de la dernière release dès que
// l'API répond, pour ne jamais pointer vers un fichier obsolète.
// Je cumule aussi les téléchargements de toutes les releases depuis
// la première : compteur total dans le hero + compteur par format
function refreshDownloadLinks() {
  const matchers = {
    deb: (name) => name.endsWith(".deb"),
    rpm: (name) => name.endsWith(".rpm"),
    dmg: (name) => name.endsWith(".dmg"),
    "setup-exe": (name) => name.endsWith("-setup.exe"),
  };
  const fmt = new Intl.NumberFormat("fr-FR");
  fetch("https://api.github.com/repos/TOS-team/Toole/releases?per_page=100")
    .then((r) => {
      if (!r.ok) throw new Error(`GitHub API ${r.status}`);
      return r.json();
    })
    .then((releases) => {
      if (!Array.isArray(releases) || releases.length === 0) {
        throw new Error("aucune release");
      }
      const latest = releases[0];
      const tag = latest.tag_name;
      const perAsset = {};
      let total = 0;
      for (const release of releases) {
        for (const asset of release.assets || []) {
          if (asset.name === "latest.json" || asset.name.endsWith(".sig")) {
            continue;
          }
          perAsset[asset.name] =
            (perAsset[asset.name] || 0) + asset.download_count;
          total += asset.download_count;
        }
      }
      for (const [key, match] of Object.entries(matchers)) {
        const asset = (latest.assets || []).find((a) => match(a.name));
        const link = document.querySelector(`a[data-asset="${key}"]`);
        if (link && asset) {
          link.href = `https://github.com/TOS-team/Toole/releases/download/${tag}/${asset.name}`;
        }
        let count = 0;
        for (const name in perAsset) {
          if (match(name)) count += perAsset[name];
        }
        const badge = document.querySelector(`[data-count="${key}"]`);
        if (badge && count > 0) {
          badge.textContent = `${fmt.format(count)} téléchargements`;
        }
      }
      const totalEl = document.getElementById("total-downloads");
      if (totalEl && total > 0) {
        document.getElementById("total-downloads-count").textContent =
          fmt.format(total);
        totalEl.classList.remove("hidden");
      }
    })
    .catch(() => {
      // API injoignable : je garde les liens de la release courante
    });
}
document.addEventListener("DOMContentLoaded", refreshDownloadLinks);

// Animations existantes
const prefersReduced = window.matchMedia(
  "(prefers-reduced-motion: reduce)",
).matches;

if (prefersReduced) {
  document
    .querySelectorAll(".reveal-text")
    .forEach((el) => el.classList.add("active"));
} else {
  setTimeout(() => {
    document
      .querySelectorAll(".reveal-text")
      .forEach((el) => el.classList.add("active"));
  }, 100);
}

if (prefersReduced) {
  document
    .querySelectorAll(".fade-up")
    .forEach((el) => el.classList.add("visible"));
} else {
  const observer = new IntersectionObserver(
    (entries) => {
      entries.forEach((entry) => {
        if (entry.isIntersecting) {
          entry.target.classList.add("visible");
          observer.unobserve(entry.target);
        }
      });
    },
    { threshold: 0.1, rootMargin: "0px 0px -50px 0px" },
  );
  document.querySelectorAll(".fade-up").forEach((el) => observer.observe(el));
}

const canvas = document.getElementById("shader-bg");
const gl = canvas && canvas.getContext("webgl");

if (gl) {
  const vertexShaderSource = `
        attribute vec2 position;
        varying vec2 v_texCoord;
        void main() {
          gl_Position = vec4(position, 0.0, 1.0);
          v_texCoord = position * 0.5 + 0.5;
        }
      `;
  const fragmentShaderSource = `
        precision highp float;
        varying vec2 v_texCoord;
        uniform float u_time;
        uniform vec2 u_resolution;
        float noise(vec2 p) {
            return fract(sin(dot(p, vec2(12.9898, 78.233))) * 43758.5453);
        }
        void main() {
            vec2 uv = v_texCoord;
            vec2 p = (gl_FragCoord.xy * 2.0 - u_resolution.xy) / min(u_resolution.x, u_resolution.y);
            vec3 color = vec3(0.02, 0.03, 0.06);
            float t = u_time * 0.4;
            vec2 cyanPos = vec2(sin(t * 0.7) * 0.5, cos(t * 0.5) * 0.3);
            float cyanGlow = 0.08 / length(p - cyanPos);
            color += vec3(0.0, 0.95, 1.0) * cyanGlow * (0.6 + 0.4 * sin(u_time));
            vec2 redPos = vec2(cos(t * 0.8) * 0.6, sin(t * 0.6) * 0.4);
            float redGlow = 0.05 / length(p - redPos);
            color += vec3(1.0, 0.0, 0.2) * redGlow * (0.5 + 0.5 * cos(u_time * 1.2));
            float grain = noise(uv * u_time) * 0.03;
            color += grain;
            gl_FragColor = vec4(color, 1.0);
        }
      `;
  function compileShader(gl, type, source) {
    const shader = gl.createShader(type);
    gl.shaderSource(shader, source);
    gl.compileShader(shader);
    if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
      console.error(gl.getShaderInfoLog(shader));
      gl.deleteShader(shader);
      return null;
    }
    return shader;
  }
  const program = gl.createProgram();
  gl.attachShader(
    program,
    compileShader(gl, gl.VERTEX_SHADER, vertexShaderSource),
  );
  gl.attachShader(
    program,
    compileShader(gl, gl.FRAGMENT_SHADER, fragmentShaderSource),
  );
  gl.linkProgram(program);
  gl.useProgram(program);
  const positionBuffer = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);
  gl.bufferData(
    gl.ARRAY_BUFFER,
    new Float32Array([-1, -1, 1, -1, -1, 1, -1, 1, 1, -1, 1, 1]),
    gl.STATIC_DRAW,
  );
  const positionLocation = gl.getAttribLocation(program, "position");
  gl.enableVertexAttribArray(positionLocation);
  gl.vertexAttribPointer(positionLocation, 2, gl.FLOAT, false, 0, 0);
  const timeLocation = gl.getUniformLocation(program, "u_time");
  const resolutionLocation = gl.getUniformLocation(program, "u_resolution");
  function resizeCanvas() {
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;
    gl.viewport(0, 0, canvas.width, canvas.height);
    gl.uniform2f(resolutionLocation, canvas.width, canvas.height);
  }
  window.addEventListener("resize", resizeCanvas);
  resizeCanvas();
  if (prefersReduced) {
    gl.uniform1f(timeLocation, 0);
    gl.drawArrays(gl.TRIANGLES, 0, 6);
  } else {
    function renderBackground(time) {
      gl.uniform1f(timeLocation, time * 0.001);
      gl.drawArrays(gl.TRIANGLES, 0, 6);
      requestAnimationFrame(renderBackground);
    }
    requestAnimationFrame(renderBackground);
  }
}

const container = document.getElementById("hero-canvas-container");
if (container && typeof THREE !== "undefined") {
  const width = container.clientWidth || window.innerWidth;
  const height = container.clientHeight || window.innerHeight;
  const scene = new THREE.Scene();
  const camera = new THREE.PerspectiveCamera(75, width / height, 0.1, 1000);
  const renderer = new THREE.WebGLRenderer({
    alpha: true,
    antialias: true,
  });
  renderer.setSize(width, height);
  renderer.setPixelRatio(window.devicePixelRatio);
  container.appendChild(renderer.domElement);
  const coreGeom = new THREE.IcosahedronGeometry(1, 1);
  const coreMat = new THREE.MeshPhongMaterial({
    color: 0x00f2ff,
    wireframe: true,
    emissive: 0x00f2ff,
    emissiveIntensity: 0.5,
  });
  const core = new THREE.Mesh(coreGeom, coreMat);
  scene.add(core);
  const group = new THREE.Group();
  const nodeGeom = new THREE.SphereGeometry(0.05, 16, 16);
  const nodeMat = new THREE.MeshBasicMaterial({ color: 0xffffff });
  for (let i = 0; i < 12; i++) {
    const node = new THREE.Mesh(nodeGeom, nodeMat);
    const angle = (i / 12) * Math.PI * 2;
    const radius = 2 + Math.random() * 0.5;
    node.position.set(
      Math.cos(angle) * radius,
      Math.sin(angle) * radius,
      (Math.random() - 0.5) * 1,
    );
    group.add(node);
  }
  scene.add(group);
  const ambientLight = new THREE.AmbientLight(0xffffff, 0.5);
  scene.add(ambientLight);
  const pointLight = new THREE.PointLight(0xff0033, 1, 10);
  pointLight.position.set(2, 2, 2);
  scene.add(pointLight);
  camera.position.z = 5;
  let mouseX = 0,
    mouseY = 0;
  document.addEventListener("mousemove", (e) => {
    mouseX = (e.clientX / window.innerWidth) * 2 - 1;
    mouseY = -(e.clientY / window.innerHeight) * 2 + 1;
  });
  function renderStatic() {
    renderer.render(scene, camera);
  }
  function animateHero() {
    requestAnimationFrame(animateHero);
    core.rotation.x += 0.005;
    core.rotation.y += 0.005;
    group.rotation.z += 0.002;
    group.rotation.y += 0.001;
    camera.position.x += (mouseX * 0.5 - camera.position.x) * 0.05;
    camera.position.y += (mouseY * 0.5 - camera.position.y) * 0.05;
    camera.lookAt(scene.position);
    renderer.render(scene, camera);
  }
  window.addEventListener("resize", () => {
    if (container) {
      const w = container.clientWidth || window.innerWidth;
      const h = container.clientHeight || window.innerHeight;
      renderer.setSize(w, h);
      camera.aspect = w / h;
      camera.updateProjectionMatrix();
    }
  });
  if (prefersReduced) {
    renderStatic();
  } else {
    animateHero();
  }
}
