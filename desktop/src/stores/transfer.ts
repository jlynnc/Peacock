import { defineStore } from "pinia";
import { invoke } from "@tauri-apps/api/core";
import {
  acceptTransfer as ipcAccept,
  rejectTransfer as ipcReject,
  pauseTransfer as ipcPause,
  resumeTransfer as ipcResume,
  cancelTransfer as ipcCancel,
} from "@/utils/ipc";

/**
 * Transfer store - now only handles transfer actions (accept/reject/pause/resume/cancel).
 * Display is handled by chat store and ChatFileCard component.
 */
export const useTransferStore = defineStore("transfer", () => {
  async function acceptOffer(transferId: string) {
    await ipcAccept(transferId);
  }

  async function acceptOfferToDir(transferId: string, dir: string) {
    await invoke("accept_transfer_to_dir", { transferId, dir });
  }

  async function rejectOffer(transferId: string) {
    await ipcReject(transferId);
  }

  async function pauseTransfer(transferId: string) {
    await ipcPause(transferId);
  }

  async function resumeTransfer(transferId: string) {
    await ipcResume(transferId);
  }

  async function cancelTransfer(transferId: string) {
    await ipcCancel(transferId);
  }

  return {
    acceptOffer,
    acceptOfferToDir,
    rejectOffer,
    pauseTransfer,
    resumeTransfer,
    cancelTransfer,
  };
});
