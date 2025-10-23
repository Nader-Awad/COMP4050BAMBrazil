import React from "react";
import ImageGallery from "../ImageGallery";

const images = [
  { src: "https://placekitten.com/800/400", alt: "Cat 1", caption: "Cat 1" },
  { src: "https://placekitten.com/801/400", alt: "Cat 2", caption: "Cat 2" },
  { src: "https://placekitten.com/802/400", alt: "Cat 3", caption: "Cat 3" },
];

export default {
  title: "UI/ImageGallery",
  component: ImageGallery,
};

export const Default = () => <ImageGallery images={images} />;