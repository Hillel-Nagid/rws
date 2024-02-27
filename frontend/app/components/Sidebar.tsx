"use client";
import React, { useState } from "react";
import { s3Links } from "../utils/helpers";
import Link from "next/link";

export default function sidebar() {
  const [selectedLink, setSelectedLink] = useState<string>("");
  return (
    <nav className="min-w-[350px] p-5 bg-gray-100">
      <div className="p-3">
        <div className="flex flex-col gap-5">
          {s3Links.map((link) => (
            <Link
              key={link.name}
              href={link.href}
              className={`w-full flex items-center gap-2`}
            >
              {" "}
              <link.Icon size={22} />{" "}
              <span className="text-normal">{link.name}</span>
            </Link>
          ))}
        </div>
      </div>
    </nav>
  );
}
