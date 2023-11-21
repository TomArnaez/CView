import { ImageMetadata, Point } from "../../bindings";

type OverlayProps = {
  pos: Point;
  adu: number;
  imageIdx: number;
  metadata: ImageMetadata;
};

const Overlay = ({ pos, adu, imageIdx, metadata }: OverlayProps) => {
  const renderExposureTime = () => {
    if (metadata.capture_settings) {
      return (
        <>
          <br />
          Image Exposure Time: {metadata.capture_settings.exp_time}ms
        </>
      );
    }
    return null;
  };

  const renderSignalAccumulationData = () => {
    if (
      metadata.extra_info &&
      metadata.extra_info.type === "SignalAccumulationData"
    ) {
      return (
        <>
          <br />
          Accumulated Exposure Time: {metadata.extra_info.accumulated_exp_time}ms
        </>
      );
    }
    return null;
  };

  const renderSmartCaptureData = () => {
    if (
      metadata.extra_info &&
      metadata.extra_info.type === "SmartCaptureData"
    ) {
      return (
        <>
          <br />
          SNR: {metadata.extra_info.signal_noise_ratio.toFixed(3)}
        </>
      );
    }
    return null;
  };

  return (
    <div
      style={{
        position: "absolute",
        top: 0,
        left: 5,
        zIndex: 999,
        color: "red",
        textAlign: "left",
      }}
    >
      Frame: {imageIdx}
      <br />
      X: {pos.x}, Y: {pos.y}
      <br />
      Saturation Level: {adu}
      {renderExposureTime()}
      {renderSignalAccumulationData()}
      {renderSmartCaptureData()}
    </div>
  );
};

export default Overlay;
