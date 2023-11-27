import { create } from "zustand";
import {
  AdvancedCapture,
  CaptureManagerStatus,
  CaptureProgress,
  LiveCapture,
  commands,
} from "../bindings";

interface DetectorState {
  darkMaps: number[];
  status: CaptureManagerStatus;
  captureProgress: CaptureProgress | null;

  setDarkMaps: (darkMaps: number[]) => void;
  setStatus: (status: CaptureManagerStatus) => void;
  setCaptureProgress: (progress: CaptureProgress) => void;
  goLive: () => void;
  runCapture: (capture: AdvancedCapture) => void;
}

export const useDetectorStore = create<DetectorState>((set) => ({
  darkMaps: [],
  status: "DetectorDisconnected",
  captureProgress: null,

  setDarkMaps: (darkMaps: number[]) => set({ darkMaps }),
  setStatus: (status: CaptureManagerStatus) => set({ status }),
  setCaptureProgress: (captureProgress: CaptureProgress) =>
    set({ captureProgress }),
  goLive: () => {
    return async () => {
      const capture: LiveCapture = {
        exp_time: 100,
        type: "LiveCapture",
      };
      await commands.runCapture(capture, false);
    };
  },
  runCapture: (capture: AdvancedCapture) => {
    return async () => {
      await commands.runCapture(capture, false);
    };
  },
}));
