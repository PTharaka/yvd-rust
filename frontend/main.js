import { invoke } from "@tauri-apps/api/core";
import { relaunch } from "@tauri-apps/plugin-process";
import { check } from "@tauri-apps/plugin-updater";

const url = document.querySelector("#url");
const queue = document.querySelector("#queue");
const message = document.querySelector("#message");
const status = document.querySelector("#status");

async function refreshQueue() {
  try {
    const tasks = await invoke("queue_status");
    queue.innerHTML = tasks.length
      ? tasks.map((task) => `
        <article class="queue-item">
          <div>
            <strong>${escapeHtml(task.id)}</strong>
            <span>${escapeHtml(task.status)}</span>
            <p>${escapeHtml(task.url)}</p>
          </div>
          <div class="actions">
            ${task.status === "queued" || task.status === "paused" || task.status === "failed"
              ? `<button data-action="start" data-id="${task.id}">START</button>` : ""}
            ${task.status === "downloading"
              ? `<button data-action="pause" data-id="${task.id}" class="secondary">PAUSE</button>` : ""}
            ${task.status === "downloading"
              ? `<button data-action="cancel" data-id="${task.id}" class="secondary">CANCEL</button>` : ""}
            ${task.status === "failed"
              ? `<button data-action="retry" data-id="${task.id}" class="secondary">RETRY</button>` : ""}
            <div class="progress-track">
              <div class="progress-bar" style="width: ${Math.max(0, Math.min(100, task.progress_percent))}%"></div>
            </div>
            <small>${Math.round(task.progress_percent)}%${task.speed ? ` · ${escapeHtml(task.speed)}` : ""}${task.eta ? ` · ETA ${escapeHtml(task.eta)}` : ""}</small>
          </div>
        </article>`).join("")
      : "<p>Queue is empty.</p>";
    status.textContent = "READY";
  } catch (error) {
    message.textContent = String(error);
    status.textContent = "ERROR";
  }
}

function escapeHtml(value) {
  return String(value).replace(/[&<>"']/g, (character) => ({
    "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;"
  }[character]));
}

queue.addEventListener("click", async (event) => {
  const button = event.target.closest("button[data-action]");
  if (!button) return;
  try {
    await invoke(`${button.dataset.action}_download`, { id: button.dataset.id });
    await refreshQueue();
  } catch (error) {
    message.textContent = String(error);
  }
});

document.querySelector("#add").addEventListener("click", async () => {
  try {
    const id = await invoke("queue_add", { url: url.value });
    message.textContent = `Added ${id} to the queue.`;
    url.value = "";
    await refreshQueue();
  } catch (error) {
    message.textContent = String(error);
  }
});
document.querySelector("#refresh").addEventListener("click", refreshQueue);
document.querySelector("#check-updates").addEventListener("click", async () => {
  const updateButton = document.querySelector("#check-updates");
  updateButton.disabled = true;
  try {
    const update = await check();
    if (!update) {
      message.textContent = "You are using the latest version.";
      return;
    }
    message.textContent = `Installing ${update.version}...`;
    await update.downloadAndInstall();
    await relaunch();
  } catch (error) {
    message.textContent = `Update failed: ${String(error)}`;
  } finally {
    updateButton.disabled = false;
  }
});
refreshQueue();
setInterval(refreshQueue, 2000);
