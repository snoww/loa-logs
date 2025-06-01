import { invoke } from "@tauri-apps/api";
import html2canvas from "html2canvas-pro";
import { addToast } from "./components/Toaster.svelte";
import { screenshot } from "./stores.svelte";
import { screenshotError, screenshotSuccess } from "./utils/toasts";

export async function takeScreenshot(div?: HTMLElement) {
  if (!div) {
    return;
  }
  screenshot.take();
  setTimeout(async () => {
    const canvas = await html2canvas(div, { useCORS: true, backgroundColor: "#27272A" });
    canvas.toBlob(async (blob) => {
      if (!blob) return;
      try {
        const item = new ClipboardItem({ "image/png": blob });
        await navigator.clipboard.write([item]);
        addToast(screenshotSuccess);
      } catch (error) {
        addToast(screenshotError);
        invoke("write_log", { message: "failed to take screenshot" });
      } finally {
        screenshot.done();
      }
    });
  }, 100);
}
