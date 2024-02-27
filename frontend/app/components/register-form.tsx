"use client";

import axios from "axios";
import Link from "next/link";
import { FormEvent, useState } from "react";
import validator from "validator";
// import { useAuth } from "../context/authContext";
import { useRouter } from "next/navigation";

export default function RegisterForm() {
  const router = useRouter();
  const [error, setError] = useState<IRegisterError>({
    email: null,
    password: null,
    username: null,
  });
  const [register, setRegister] = useState<IRegister>({
    email: "",
    password: "",
    username: "",
  });

  function submitForm(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();
    if (!validator.isEmail(register.email)) {
      setError((prev) => ({ ...prev, email: "Invalid Email Address" }));
    }
    if (!validator.isStrongPassword(register.password)) {
      setError((prev) => ({ ...prev, password: "Password not strong enough" }));
    }
    if (register.username.length <= 2) {
      setError((prev) => ({
        ...prev,
        username: "Username must be a least 2 character",
      }));
    }

    if (register.username.includes("@") || register.username.includes(".")) {
      setError((prev) => ({
        ...prev,
        username: "Username can not contains the characters: @ or .",
      }));
    }

    if (!error.email && !error.username && !error.password) {
      try {
        var bodyFormData = new FormData();
        bodyFormData.append("email", register.email);
        bodyFormData.append("password", register.password);
        bodyFormData.append("username", register.username);
        axios
          .post("http://localhost:2945/signup", bodyFormData)
          .then((res) => {
            setError((prev) => ({ ...prev, email: null }));
            setError((prev) => ({ ...prev, password: null }));
            setError((prev) => ({ ...prev, username: null }));
            router.push("/");
          })
          .catch((err) => {
            setError(err.response?.data.message || "Something went wrong 😢");
            console.log(err);
          });
      } catch (error) {
        setError.prototype.email("Something went wrong 😢");
        console.log(error);
      }
    }
  }

  return (
    <div className="flex h-screen flex-1 flex-col justify-center px-6 py-12 lg:px-8">
      <div className="sm:mx-auto sm:w-full sm:max-w-sm">
        <p className="text-center h-14 text-white mx-auto w-14 flex items-center justify-center rounded-full  bg-indigo-600 font-semibold">
          RWS
        </p>
        <h2 className="mt-10 text-center text-2xl font-bold leading-9 tracking-tight text-gray-900">
          Create an account
        </h2>
      </div>

      <div className="mt-10 sm:mx-auto sm:w-full sm:max-w-sm">
        {/* <form className="space-y-6" action="#" method="POST"> */}
        <form onSubmit={submitForm} className="space-y-6">
          <div>
            <label
              htmlFor="email"
              className="block text-sm font-medium leading-6 text-gray-900"
            >
              Email address
            </label>
            <div className="mt-2">
              <input
                value={register.email}
                onChange={(e) =>
                  setRegister((prev) => ({ ...prev, email: e.target.value }))
                }
                id="email"
                name="email"
                type="email"
                autoComplete="email"
                required
                className="block w-full rounded-md border-0 py-1.5 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-indigo-600 sm:text-sm sm:leading-6"
              />
            </div>
            {error.email && <span>{error.email}</span>}
          </div>

          <div>
            <label
              htmlFor="email"
              className="block text-sm font-medium leading-6 text-gray-900"
            >
              Username
            </label>
            <div className="mt-2">
              <input
                value={register.username}
                onChange={(e) =>
                  setRegister((prev) => ({ ...prev, username: e.target.value }))
                }
                id="username"
                name="username"
                type="text"
                autoComplete="username"
                // autoComplete="email"
                required
                className="block w-full rounded-md border-0 py-1.5 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-indigo-600 sm:text-sm sm:leading-6"
              />
            </div>
            {error.username && <span>{error.username}</span>}
          </div>

          <div>
            <div className="flex items-center justify-between">
              <label
                htmlFor="password"
                className="block text-sm font-medium leading-6 text-gray-900"
              >
                Password
              </label>
            </div>
            <div className="mt-2">
              <input
                value={register.password}
                onChange={(e) =>
                  setRegister((prev) => ({ ...prev, password: e.target.value }))
                }
                id="password"
                name="password"
                type="password"
                autoComplete="current-password"
                required
                minLength={8}
                className="block w-full rounded-md border-0 py-1.5 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-indigo-600 sm:text-sm sm:leading-6"
              />
            </div>
            {error.password && <span>{error.password}</span>}
          </div>

          <div>
            <button
              type="submit"
              className="flex w-full justify-center rounded-md bg-indigo-600 px-3 py-1.5 text-sm font-semibold leading-6 text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600"
            >
              Register
            </button>
          </div>
        </form>

        <p className="mt-10 text-center text-sm text-gray-500">
          Already a member?{" "}
          <Link
            href="/signin"
            className="font-semibold leading-6 text-indigo-600 hover:text-indigo-500"
          >
            Log in
          </Link>
        </p>
      </div>
    </div>
  );
}
