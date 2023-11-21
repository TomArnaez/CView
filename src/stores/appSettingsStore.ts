import { create } from "zustand"

interface AppSettings {
    saturatedPixelRGBColour: string,
    updatedPixelRGBColour: (newColour: string) => void,
    saturatedPixelThreshold: number,
    updateSaturatedPixelThreshold: (newThreshold: number) => void,
    autoSaveCaptures: boolean,
    updateAutoSaveCaptures: (newValue: boolean) => void
}

export const useAppSettingsStore = create<AppSettings>( (set) => ({
    saturatedPixelRGBColour: "red",
    updatedPixelRGBColour: (newColour: string) => set({ saturatedPixelRGBColour: newColour}),
    saturatedPixelThreshold: 16000,
    updateSaturatedPixelThreshold: (newThreshold: number) => set(({saturatedPixelThreshold: newThreshold})),
    autoSaveCaptures: true,
    updateAutoSaveCaptures: (newValue: boolean) => set({ autoSaveCaptures: newValue }),
}));