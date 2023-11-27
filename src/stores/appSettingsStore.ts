import { create } from "zustand";

interface AppSettings {
  saturatedPixelRGBColour: string;
  setSaturatedPixelRGBColour: (newColour: string) => void;
  saturatedPixelThreshold: number;
  setSaturatedPixelThreshold: (newThreshold: number) => void;
  autoSaveCaptures: boolean;
  setAutoSaveCaptures: (newValue: boolean) => void;
  autoHistogramEqualization: boolean;
  setAutoHistogramEqualization: (newValue: boolean) => void;
}

export const useAppSettingsStore = create<AppSettings>((set) => ({
  saturatedPixelRGBColour: "rgb(250, 0, 0)",
  setSaturatedPixelRGBColour: (newColour: string) => {
    console.log(newColour);
    set({ saturatedPixelRGBColour: newColour });
  },
  saturatedPixelThreshold: 16000,
  setSaturatedPixelThreshold: (newThreshold: number) =>
    set({ saturatedPixelThreshold: newThreshold }),
  autoSaveCaptures: false,
  setAutoSaveCaptures: (newValue: boolean) =>
    set({ autoSaveCaptures: newValue }),
  autoHistogramEqualization: true,
  setAutoHistogramEqualization: (newValue: boolean) =>
    set({ autoHistogramEqualization: newValue }),
}));
