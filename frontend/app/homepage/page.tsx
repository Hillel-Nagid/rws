"use client";
import React, { useEffect } from "react";
import { useRouter } from "next/navigation";
import { popularProducts } from "../utils/helpers";
import Link from "next/link";
import axios from "axios";

export default function page() {
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

  useEffect(() => {
    try {
      // var bodyFormData = new FormData();
      // bodyFormData.append("identifier", user.usernameOrEmail);
      // bodyFormData.append("password", user.password);

    //   axios
    //     .post("http://localhost:2945/signin", bodyFormData)
    //     .then((res) => {
    //       login(res.data.user, res.data.token);
    //       setError({
    //         captcha: null,
    //         password: null,
    //         usernameOrEmail: null,
    //       });
    //       router.push("/homepage");
    //     })
    //     .catch((err) => {
    //       console.log(err);
    //       setError({
    //         usernameOrEmail: err.response?.data.message || "Something went wrong 😢",
    //         password: null,
    //         captcha: null
    //       });
    //     });
    }
     catch (error) {
    //   setError.prototype.usernameOrEmail("Something went wrong 😢");
    //   console.log(error);
    }
  }, []);

  return (
    <>
      <div>Home Page</div>
      <div>
        <h3>Popular</h3>
        <div>
          {productsWithPlaceholders.map((product) =>
            product.disabled ? (
              <span
                key={product.name}
                style={{ color: "gray", textDecoration: "line-through" }}
              >
                {product.name}
              </span>
            ) : (
              <Link key={product.name} href={product.href}>
                {product.name}
              </Link>
            )
          )}
        </div>
      </div>
    </>
  );
}
