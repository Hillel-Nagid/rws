"use client";
import React, { useEffect } from "react";
import { useRouter } from "next/navigation";

type Props = {};

export default function S3({}: Props) {
  const router = useRouter();
  // Access token from AuthContext
  let token;
  useEffect(() => {
    // Check if localStorage is available
    if (typeof window !== "undefined") {
      token = localStorage.getItem("token");
      if (!token) {
        router.push("/signin");
      }
    }
  }, [token, router]);
  return (
    <div>
      <h1>Choose A Bucket</h1>
      {/* get buckets that yoou have permission to see (filter the ones with an error) */}
      {/* {display it here} */}
    </div>
  );
}
