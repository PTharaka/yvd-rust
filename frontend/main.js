import { invoke } from "@tauri-apps/api/core";
import { relaunch } from "@tauri-apps/plugin-process";
import { check } from "@tauri-apps/plugin-updater";

const url = document.querySelector("#url");
const queue = document.querySelector("#queue");
const message = document.querySelector("#message");
const status = document.querySelector("#status");
const preview = document.querySelector("#preview");
const thumbnail = document.querySelector("#thumbnail");
const previewTitle = document.querySelector("#preview-title");
const previewChannel = document.querySelector("#preview-channel");
const previewDetails = document.querySelector("#preview-details");
let previewUrl = "";

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

async function loadPreview() {
  const value = url.value.trim();
  if (!value) {
    message.textContent = "Enter a video or playlist URL first.";
    return;
  }
  status.textContent = "LOADING";
  message.textContent = "Loading media details...";
  try {
    const metadata = await invoke("metadata_preview", { url: value });
    previewUrl = value;
    thumbnail.src = metadata.thumbnail || "";
    thumbnail.hidden = !metadata.thumbnail;
    previewTitle.textContent = metadata.title || "Untitled media";
    previewChannel.textContent = metadata.channel ? `Channel: ${metadata.channel}` : "";
    previewDetails.textContent = metadata.item_count
      ? `Playlist: ${metadata.item_count} item(s)`
      : `Duration: ${formatDuration(metadata.duration_seconds)}`;
    preview.hidden = false;
    message.textContent = "Review the media details, then add it to the queue.";
    status.textContent = "READY";
  } catch (error) {
    preview.hidden = true;
    message.textContent = `Metadata lookup failed: ${String(error)}`;
    status.textContent = "ERROR";
  }
}

document.querySelector("#add").addEventListener("click", loadPreview);
url.addEventListener("change", loadPreview);
document.querySelector("#add-preview").addEventListener("click", async () => {
  if (!previewUrl) return;
  try {
    const id = await invoke("queue_add", { url: previewUrl });
    message.textContent = `Added ${id} to the queue.`;
    url.value = "";
    preview.hidden = true;
    previewUrl = "";
    await refreshQueue();
  } catch (error) {
    message.textContent = String(error);
  }
});
document.querySelector("#clear-preview").addEventListener("click", () => {
  preview.hidden = true;
  previewUrl = "";
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

function formatDuration(seconds) {
  if (!Number.isFinite(seconds)) return "Unknown duration";
  const minutes = Math.floor(seconds / 60);
  return `${minutes}:${String(Math.floor(seconds % 60)).padStart(2, "0")}`;
}
