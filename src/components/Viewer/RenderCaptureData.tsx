import { Group, Rect, Text } from "react-konva";
import { CaptureResultData } from "../../bindings";

export const renderCaptureData = (
  extraData: CaptureResultData | null
): React.ReactElement | null => {
  if (extraData == null) {
    return null;
  }
  switch (extraData.type) {
    case "SmartCaptureData":
      return (
        <Group>
          <Rect
            x={extraData.background_rect.pos.x}
            y={extraData.background_rect.pos.y}
            width={extraData.background_rect.width}
            height={extraData.background_rect.height}
            stroke={"blue"}
            strokeWidth={1}
          ></Rect>
           <Text x={extraData.background_rect.pos.x} y={extraData.background_rect.pos.y - 10} text="Background Window" fontSize={7} fill="red"/>        
          <Rect
            x={extraData.foreground_rect.pos.x}
            y={extraData.foreground_rect.pos.y}
            width={extraData.foreground_rect.width}
            height={extraData.foreground_rect.height}
            stroke={"blue"}
            strokeWidth={1}
          ></Rect>
            <Text x={extraData.foreground_rect.pos.x} y={extraData.foreground_rect.pos.y - 10} text="Foreground Window" fontSize={7} fill="red"/>        
          </Group>
      );
    case "SignalAccumulationData": {
      return null;
    }
    default:
      return null;
  }
};
