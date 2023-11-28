import { Swiper, SwiperSlide } from "swiper/react";
import "swiper/css";
import { useImageStore } from "../../stores/imageStore";

const Carousel = () => {
  const { currentStackIdx, imageStacks, setCurrentImageIdx } = useImageStore((state) => ({
    currentStackIdx: state.currentStackIndex,
    imageStacks: state.imageStacks,
    setCurrentImageIdx: state.setImage
  }));

  return (
    <Swiper
      width={600}
      height={100}
      spaceBetween={10}
      slidesPerView={3}
      navigation={true}
      freeMode={true}
    >
      {currentStackIdx < imageStacks.length && imageStacks[currentStackIdx].image_handlers.map((handler, index) => {
        return (
          <SwiperSlide onDoubleClick={() => console.log("double click", index)}>
            <img src="https://swiperjs.com/demos/images/nature-1.jpg" />
          </SwiperSlide>
        );
      })}
    </Swiper>
  );
};

export default Carousel;
