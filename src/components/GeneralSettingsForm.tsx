import {
  Center,
  Checkbox,
  ColorInput,
  NumberInput,
  Title,
} from "@mantine/core";
import { useAppSettingsStore } from "../stores/appSettingsStore";

export const GeneralSettingsForm = (): JSX.Element => {
  const {
    saturatedPixelColor,
    saturatedPixelThreshold,
    autoSaveCaptures,
    setSaturatedPixelColour,
    setdSaturatedPixelThreshold,
    setAutoSaveCaptures,
  } = useAppSettingsStore((state) => ({
    saturatedPixelThreshold: state.saturatedPixelThreshold,
    saturatedPixelColor: state.saturatedPixelRGBColour,
    autoSaveCaptures: state.autoSaveCaptures,
    setSaturatedPixelColour: state.setSaturatedPixelRGBColour,
    setdSaturatedPixelThreshold: state.setSaturatedPixelThreshold,
    setAutoSaveCaptures: state.setAutoSaveCaptures,
  }));

  return (
    <>
      <Center>
        <Title order={4}>Application Settings</Title>
      </Center>
      <br />
      <Checkbox
        label="Auto-Save Captures"
        checked={autoSaveCaptures}
        onChange={() => setAutoSaveCaptures(!autoSaveCaptures)}
      ></Checkbox>
      <ColorInput
        label="Set colour for saturated pixels"
        value={saturatedPixelColor}
        format="rgb"
        onChange={(color: string) => {
          setSaturatedPixelColour(color);
        }}
      />
      <NumberInput
        label="Enter the threshold for considering a pixel saturated"
        defaultValue={saturatedPixelThreshold}
        onChange={(threshold: number) => {
          setdSaturatedPixelThreshold(threshold);
        }}
        min={0}
        max={16384}
      />
    </>
  );
};
