import { ImageStack } from "../bindings";

export type Image = {
  data: Uint16Array;
  width: number;
  height: number;
}

export interface ImageState {
  imageStacks: ImageStack[];
  currentStackIndex: number;
  currentImageIndex: number;
}

export type ImageStateAction =
  | { type: "INCREMENT_IMAGE_INDEX" }
  | { type: "DECREMENT_IMAGE_INDEX" }
  | { type: "SET_STACK_INDEX"; index: number }
  | { type: "SET_IMAGE_INDEX"; index: number }
  | { type: "UPDATE_IMAGE_STATE"; imageStacks: ImageStack[] };
