import { Rect, Text } from "react-konva";
import { CaptureResultData } from "../../bindings";

export const renderCaptureData = (
  extraData: CaptureResultData | null,
  sceneScale: number
): React.ReactElement | null => {
  if (extraData == null) {
    return null;
  }
  switch (extraData.type) {
    case "SmartCaptureData":
      console.log(extraData);
      return (
        <>
          <Rect
            x={extraData.background_rect.pos.x * sceneScale}
            y={extraData.background_rect.pos.y * sceneScale}
            width={extraData.background_rect.width * sceneScale}
            height={extraData.background_rect.height * sceneScale}
            stroke={"blue"}
            strokeWidth={1}
          ></Rect>
          <Text
            x={extraData.background_rect.pos.x * sceneScale}
            y={(extraData.background_rect.pos.y - 10) * sceneScale}
            text="Background Window"
            fontSize={7 * sceneScale}
            fill="red"
          />
          <Rect
            x={extraData.foreground_rect.pos.x * sceneScale}
            y={extraData.foreground_rect.pos.y * sceneScale}
            width={extraData.foreground_rect.width * sceneScale}
            height={extraData.foreground_rect.height * sceneScale}
            stroke={"blue"}
            strokeWidth={1}
          ></Rect>
          <Text
            x={extraData.foreground_rect.pos.x * sceneScale}
            y={(extraData.foreground_rect.pos.y - 10) * sceneScale}
            text="Foreground Window"
            fontSize={7 * sceneScale}
            fill="red"
          />
        </>
      );
    case "SignalAccumulationData": {
      return null;
    }
    default:
      return null;
  }
};
