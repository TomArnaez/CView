import { create } from "zustand";
import { ImageMetadata, ImageStack } from "../bindings";
import { Image } from "../types/imagestate";
import { invoke } from "@tauri-apps/api/primitives";

interface ImageState {
  imageStacks: ImageStack[];
  currentStackIndex: number;
  currentImageIndex: number;
  streaming: boolean;
  currentImageCanvas: HTMLCanvasElement | null,

  incrementImage: (by: number) => void;
  incrementStack: (by: number) => void;
  decrementImage: (by: number) => void;
  decrementStack: (by: number) => void;
  setStreaming: (by: boolean) => void;
  setStack: (idx: number) => void;
  setImage: (idx: number) => void;
  updateStacks: (newStacks: ImageStack[]) => void;
  getCurrentImage: () => Promise<Image | null>;
  getImageMetaData: (imageIdx: number, stackIdx: number) => ImageMetadata;
}

const getCurrentImageAsync = async (
  state: ImageState
): Promise<Image | null> => {
  if (state.imageStacks.length === 0) {
    return null;
  }

  const { currentStackIndex, currentImageIndex } = state;
  const data = new Uint16Array(
    await invoke("get_image_binary", {
      stack_idx: currentStackIndex,
      image_idx: currentImageIndex,
      resize: false,
    })
  );

  return { data, width: 1031, height: 1536 };
};

const isValidIndex = (index: number, length: number) =>
  index >= 0 && index < length;

export const useImageStore = create<ImageState>()((set, get) => ({
  imageStacks: [],
  currentImageIndex: 0,
  currentStackIndex: 0,
  streaming: false,
  currentImageCanvas: null,

  incrementImage: (by) =>
    set((state) => {
      const newIndex = state.currentImageIndex + by;

      if (
        isValidIndex(
          newIndex,
          get().imageStacks[state.currentStackIndex].image_handlers.length
        )
      ) {
        return { currentImageIndex: newIndex };
      }

      return state;
    }),
  setStreaming: (by) => set(() => ({ streaming: by })),
  decrementImage: (by) =>
    set((state) => {
      const newIndex = state.currentImageIndex - by;

      if (
        isValidIndex(
          newIndex,
          get().imageStacks[state.currentStackIndex].image_handlers.length
        )
      ) {
        return { currentImageIndex: newIndex };
      }

      return state;
    }),
  incrementStack: (by) =>
    set((state) => {
      const newIndex = state.currentStackIndex + by;

      if (isValidIndex(newIndex, get().imageStacks.length)) {
        return { currentImageIndex: newIndex };
      }

      return state;
    }),
  decrementStack: (by) =>
    set((state) => {
      const newIndex = state.currentStackIndex - by;

      if (isValidIndex(newIndex, get().imageStacks.length)) {
        return { currentImageIndex: newIndex };
      }

      return state;
    }), 
  setStack: (idx) => set(() => ({ currentStackIndex: idx, currentImageIndex: 0 })),
  setImage: (idx) => set(() => ({ currentImageIndex: idx })),
  updateStacks: (newStacks) => {
    // Calculate the index of the last stack
    const lastStackIndex = newStacks.length - 1;

    // Update the state with the new stacks, set current stack to last and image index to 0
    set(() => ({
      imageStacks: newStacks,
      currentStackIndex: Math.max(0, lastStackIndex), // Ensure it's not negative
      currentImageIndex: 0
    }));},  
  getCurrentImage: () => getCurrentImageAsync(get()),
  getImageMetaData: (imageIdx, stackIdx) =>
    get().imageStacks[stackIdx].image_handlers[imageIdx].image_metadata,
}));
