import { ColorInput, NumberInput } from "@mantine/core";
import { useAppSettingsStore } from "../stores/appSettingsStore";

export const GeneralSettingsForm = (): JSX.Element => {
  const {saturatedPixelColor, saturatedPixelThreshold, updateSaturatedPixelColour, updatedSaturatedPixelThreshold} = useAppSettingsStore((state) => ({
    saturatedPixelThreshold: state.saturatedPixelThreshold,
    saturatedPixelColor: state.saturatedPixelRGBColour,
    updateSaturatedPixelColour: state.updatedPixelRGBColour,
    updatedSaturatedPixelThreshold: state.updateSaturatedPixelThreshold
  }));

  return (
    <>
      <ColorInput
        label="Set colour for saturated pixels"
        defaultValue={saturatedPixelColor}
        format="rgb"
        onChange={(color: string) => {updateSaturatedPixelColour(color)}}
      />
      <NumberInput
        label="Enter the threshold for considering a pixel saturated"
        defaultValue={saturatedPixelThreshold}
        onChange={(threshold: number) => {updatedSaturatedPixelThreshold(threshold)}}
        min={0}
        max={16384}
      />
    </>
  );
};
