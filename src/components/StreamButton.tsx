import { LiveCapture, commands } from "../bindings";
import { useDetectorStore } from "../stores/detectorStore";
import { useImageStore } from "../stores/imageStore";
import { isCapturingStatus } from "../utils";

const StreamButton = () => {
  const { status } = useDetectorStore((state) => ({
    status: state.status,
  }));
  const { setStreaming } = useImageStore((state) => ({
    setStreaming: state.setStreaming,
  }));

  const isDisabled =
    status === "DetectorDisconnected" ||
    status === "DarkMapsRequired" ||
    status === "DefectMapsRequired" ||
    (typeof status === "object" && status.Capturing.type !== "LiveCapture");

  const isLiveCapture =
    typeof status === "object" && status.Capturing.type === "LiveCapture";

  const buttonClass = `relative text-lg px-4 py-2 font-semibold rounded text-white w-full h-full ${
    isDisabled
      ? "bg-gray-400 text-gray-200"
      : isLiveCapture
      ? "bg-red-500"
      : "bg-blue-500"
  }`;

  const handleClick = async () => {
    if (isCapturingStatus(status)) {
      if (status.Capturing.type == "LiveCapture") {
        await commands.stopCapture();
        setStreaming(false);
      }
    } else if (status === "Available") {
      setStreaming(true);
      const capture: LiveCapture = {
        exp_time: 100,
        type: "LiveCapture",
      };
      await commands.runCapture(capture, false);
    }
  };

  return (
    <button className={buttonClass} disabled={isDisabled} onClick={handleClick}>
      {status === "Available"
        ? "Go Live"
        : isCapturingStatus(status) && status.Capturing.type === "LiveCapture"
        ? "Stop Live"
        : "Go Live"}
    </button>
  );
};

export default StreamButton;
