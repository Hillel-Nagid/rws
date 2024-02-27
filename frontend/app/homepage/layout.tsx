import React from "react";
import Sidebar from "../components/Sidebar";

type Props = {
  children: React.ReactNode;
};

export default function layout({ children }: Props) {
  return <div className="flex items-start justify-start gap-3">
    <Sidebar/>
    {children}
    </div>;
}
