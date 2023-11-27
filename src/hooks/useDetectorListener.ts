import { useEffect } from "react";
import { useDetectorStore } from "../stores/detectorStore";
import { events } from "../bindings";

const useDetectorListener = () => {
  const { setDarkMaps, setStatus, setCaptureProgress } = useDetectorStore(
    (state) => ({
      setDarkMaps: state.setDarkMaps,
      setStatus: state.setStatus,
      setCaptureProgress: state.setCaptureProgress,
    })
  );

  useEffect(() => {
    const unsubscribeCaptureManagerEvent = events.captureManagerEvent.listen(
      async (e) => {
        setDarkMaps(e.payload.dark_maps);
        setStatus(e.payload.status);
      }
    );

    const unsubsribeCaptureProgressEvent = events.captureProgressEvent.listen(
      async (e) => {
        setCaptureProgress(e.payload);
      }
    );

    return () => {
      unsubscribeCaptureManagerEvent.then((unsub) => unsub());
      unsubsribeCaptureProgressEvent.then((unsub) => unsub());
    };
  }, [setCaptureProgress, setDarkMaps, setStatus]);
};

export default useDetectorListener;
