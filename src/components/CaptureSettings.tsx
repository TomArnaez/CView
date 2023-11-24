import {
  Button,
  NumberInput,
  Grid,
  MultiSelect,
  NativeSelect,
  Stack,
  Text,
  Checkbox,
} from "@mantine/core";
import {
  AdvancedCapture,
  MultiCapture,
  SignalAccumulationCapture,
  SmartCapture,
} from "../bindings";
import { useState } from "react";

type SignalAccumulatiorCaptureEditorProps = {
  captureMode: SignalAccumulationCapture;
  darkMapExps: number[];
  setCaptureMode: React.Dispatch<React.SetStateAction<AdvancedCapture>>;
};

const SignalAccumulatiorCaptureEditor = ({
  captureMode,
  darkMapExps,
  setCaptureMode,
}: SignalAccumulatiorCaptureEditorProps): JSX.Element => {
  return (
    <Grid>
      <Grid.Col span={4}>
        <Text>Exposure Time</Text>
      </Grid.Col>
      <Grid.Col span={8}>
        <MultiSelect
          onChange={(v) => {
            const new_exp_times: number[] = v.map((str: string) =>
              parseInt(str, 10)
            );
            setCaptureMode({ ...captureMode, exp_times: new_exp_times });
            console.log(captureMode);
          }}
          data={darkMapExps.map((expTime) => {
            return {
              value: expTime.toString(),
              label: expTime.toString() + "ms",
            };
          })}
        ></MultiSelect>
      </Grid.Col>
      <Grid.Col span={4}>
        <Text>Frame Count</Text>
      </Grid.Col>
      <Grid.Col span={8}>
        <NumberInput
          value={captureMode.frames_per_capture}
          onChange={(v) =>
            setCaptureMode({ ...captureMode, frames_per_capture: Number(v) })
          }
        />
      </Grid.Col>
    </Grid>
  );
};

type MultiCaptureEditorProps = {
  captureMode: MultiCapture;
  darkMapExps: number[];
  setCaptureMode: React.Dispatch<React.SetStateAction<AdvancedCapture>>;
};

const MultiCaptureEditor = ({
  captureMode,
  darkMapExps,
  setCaptureMode,
}: MultiCaptureEditorProps): JSX.Element => {
  return (
    <Grid>
      <Grid.Col span={4}>
        <Text>Exposure Times</Text>
      </Grid.Col>
      <Grid.Col span={8}>
        <MultiSelect
          onChange={(v) => {
            const new_exp_times: number[] = v.map((str: string) =>
              parseInt(str, 10)
            );
            setCaptureMode({ ...captureMode, exp_times: new_exp_times });
            console.log(captureMode);
          }}
          data={darkMapExps.map((expTime) => {
            return {
              value: expTime.toString(),
              label: expTime.toString() + "ms",
            };
          })}
        ></MultiSelect>
      </Grid.Col>
      <Grid.Col span={4}>
        <Text>Frames Per Capture</Text>
      </Grid.Col>
      <Grid.Col span={8}>
        <NumberInput
          value={captureMode.frames_per_capture}
          onChange={(v) =>
            setCaptureMode({ ...captureMode, frames_per_capture: Number(v) })
          }
        />
      </Grid.Col>
    </Grid>
  );
};

type SmartCaptureEditorProps = {
  captureMode: SmartCapture;
  darkMapExps: number[];
  setCaptureMode: React.Dispatch<React.SetStateAction<AdvancedCapture>>;
};

const SmartCaptureEditor = ({
  captureMode,
  darkMapExps,
  setCaptureMode,
}: SmartCaptureEditorProps): JSX.Element => {
  return (
    <Grid>
      <Grid.Col span={4}>
        <Text>Frames Per Capture</Text>
      </Grid.Col>
      <Grid.Col span={8}>
        <NumberInput
          value={captureMode.frames_per_capture}
          onChange={(v) =>
            setCaptureMode({ ...captureMode, frames_per_capture: Number(v) })
          }
        />
      </Grid.Col>
      <Grid.Col span={4}>
        <Text>Window Size</Text>
      </Grid.Col>
      <Grid.Col span={8}>
        <NumberInput
          value={captureMode.window_size}
          onChange={(v) =>
            setCaptureMode({ ...captureMode, window_size: Number(v) })
          }
        />
      </Grid.Col>
      <Grid.Col span={4}>
        <Text>Exposure Times</Text>
      </Grid.Col>
      <Grid.Col span={8}>
        <MultiSelect
          onChange={(v) => {
            const new_exp_times: number[] = v.map((str: string) =>
              parseInt(str, 10)
            );
            setCaptureMode({ ...captureMode, exp_times: new_exp_times });
          }}
          data={darkMapExps.map((expTime) => {
            return {
              value: expTime.toString(),
              label: expTime.toString() + "ms",
            };
          })}
        ></MultiSelect>
      </Grid.Col>
    </Grid>
  );
};

const editorConfiguration = {
  SignalAccumulationCapture: SignalAccumulatiorCaptureEditor,
  MultiCapture: MultiCaptureEditor,
  SmartCapture: SmartCaptureEditor,
};

type Props = {
  darkMapExps: number[];
  setFormOpen: React.Dispatch<React.SetStateAction<boolean>>;
  startCapture: any;
};

export const CaptureSettings = ({
  darkMapExps,
  startCapture,
}: Props): JSX.Element => {
  const [captureMode, setCaptureMode] = useState<AdvancedCapture>({
    exp_times: [100, 250],
    frames_per_capture: 3,
    type: "SignalAccumulationCapture",
  });

  const renderCaptureEditor = () => {
    if (captureMode && editorConfiguration[captureMode.type]) {
      const CaptureEditor = editorConfiguration[captureMode.type];
      return (
        <CaptureEditor
          captureMode={captureMode}
          darkMapExps={darkMapExps}
          setCaptureMode={setCaptureMode}
        />
      );
    } else {
      return <em>Select an element type to display</em>;
    }
  };

  const handleCaptureSelectChange = (captureMode: string) => {
    const capture = createCapture(captureMode);
    if (capture) {
      setCaptureMode(capture);
    }
  };

  const createCapture = (type: string): AdvancedCapture | undefined => {
    switch (type) {
      case "SignalAccumulationCapture":
        return {
          exp_times: [1],
          frames_per_capture: 10,
          type: "SignalAccumulationCapture",
        };
      case "MultiCapture":
        return {
          exp_times: [1],
          frames_per_capture: 1,
          type: "MultiCapture",
        };
      case "SmartCapture":
        return {
          exp_times: [1],
          frames_per_capture: 1,
          window_size: 5,
          median_filtered: false,
          type: "SmartCapture",
        };
    }
  };

  return (
    <Stack>
      <NativeSelect
        label="Capture Mode"
        onChange={(e) => {
          handleCaptureSelectChange(e.target.value);
        }}
        value={captureMode.type}
        data={[
          {
            value: "SignalAccumulationCapture",
            label: "Signal Accumulation Capture",
          },
          { value: "SmartCapture", label: "Smart Capture" },
          { value: "MultiCapture", label: "Multi Capture" },
        ]}
      />
      {renderCaptureEditor()}
      <Button onClick={() => startCapture(captureMode)}>Run Capture</Button>
    </Stack>
  );
};
