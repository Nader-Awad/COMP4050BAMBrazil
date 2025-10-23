import React from "react";
import UserAvatar from "../UserAvatar";

export default {
  title: "UI/UserAvatar",
  component: UserAvatar,
};

export const Default = () => (
  <div className="flex gap-4 items-center">
    <UserAvatar name="Ada Lovelace" />
    <UserAvatar name="Alan Turing" size="lg" />
    <UserAvatar src="https://placekitten.com/80/80" name="Kitty" />
  </div>
);