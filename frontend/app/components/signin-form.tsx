"use client";

import Link from "next/link";
import React, { FormEvent, LegacyRef, useState } from "react";
import { useAuth } from "../context/authContext";
import axios from "axios";
import { useRouter } from "next/navigation";
import ReCAPTCHA from "react-google-recaptcha";
import validator from "validator";

export default function SigninForm() {
  const [error, setError] = useState<ILoginError>({
    usernameOrEmail: null,
    password: null,
    captcha: null,
  });
  const recaptchaRef: LegacyRef<ReCAPTCHA> | undefined = React.createRef();
  const router = useRouter();
  const { login } = useAuth();
  const [user, setUser] = useState<ILogin>({
    usernameOrEmail: "",
    password: "",
  });

  function submitForm(e: FormEvent<HTMLFormElement>) {
    e.preventDefault();

    if (!validator.isEmail(user.usernameOrEmail) && user.usernameOrEmail.includes('@')) {
      setError((prev) => ({
        ...prev,
        usernameOrEmail: "Invalid Email Address",
      }));
    }
    if (!validator.isStrongPassword(user.password)) {
      setError((prev) => ({
        ...prev,
        password: "Password is not strong enough",
      }));
    }
    if (user.usernameOrEmail.length <= 2) {
      setError((prev) => ({
        ...prev,
        username: "Username must be a least 2 character",
      }));
    }
    if (!error.usernameOrEmail && !error.password && !error.captcha) {
      try {
        var bodyFormData = new FormData();
        bodyFormData.append("identifier", user.usernameOrEmail);
        bodyFormData.append("password", user.password);

        axios
          .post("http://localhost:2945/signin", bodyFormData)
          .then((res) => {
            login(res.data.user, res.data.token);
            setError({
              captcha: null,
              password: null,
              usernameOrEmail: null,
            });
            router.push("/homepage");
          })
          .catch((err) => {
            console.log(err);
            setError({
              usernameOrEmail: err.response?.data.message || "Something went wrong 😢",
              password: null,
              captcha: null
            });
          });
      } catch (error) {
        setError.prototype.usernameOrEmail("Something went wrong 😢");
        console.log(error);
      }
    }
  }

  function onReCAPTCHAChange(captchaCode: string | null) {
    setError((prevError) => ({
      ...prevError,
      captcha: captchaCode
        ? null
        : "Please complete the reCAPTCHA verification.",
    }));
  }
  return (
    <div className="flex h-screen flex-1 flex-col justify-center px-6 py-12 lg:px-8">
      <div className="sm:mx-auto sm:w-full sm:max-w-sm">
        <p className="text-center h-14 text-white mx-auto w-14 flex items-center justify-center rounded-full  bg-indigo-600 font-semibold">
          RWS
        </p>
        <h2 className="mt-10 text-center text-2xl font-bold leading-9 tracking-tight text-gray-900">
          Sign in to your account
        </h2>
      </div>

      <div className="mt-10 sm:mx-auto sm:w-full sm:max-w-sm">
        <form
          onSubmit={submitForm}
          className="space-y-6"
          action="#"
          method="POST"
        >
          <div>
            <label
              htmlFor="usernameOrEmail"
              className="block text-sm font-medium leading-6 text-gray-900"
            >
              Username or Email address
            </label>
            <div className="mt-2">
              <input
                value={user.usernameOrEmail}
                onChange={(e) =>
                  setUser((prev) => ({
                    ...prev,
                    usernameOrEmail: e.target.value,
                  }))
                }
                id="usernameOrEmail"
                name="usernameOrEmail"
                type="text"
                required
                className="block w-full rounded-md border-0 py-1.5 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-indigo-600 sm:text-sm sm:leading-6"
              />
            </div>
          </div>

          <div>
            <div className="flex items-center justify-between">
              <label
                htmlFor="password"
                className="block text-sm font-medium leading-6 text-gray-900"
              >
                Password
              </label>
              {/* <div className="text-sm">
                <Link
                  href="#"
                  className="font-semibold text-indigo-600 hover:text-indigo-500"
                >
                  Forgot password?
                </Link>
              </div> */}
            </div>
            <div className="mt-2">
              <input
                value={user.password}
                onChange={(e) => {
                  setUser((prev) => ({ ...prev, password: e.target.value }));
                  if (!validator.isStrongPassword(e.target.value)) {
                    setError((er) => ({
                      ...er,
                      password: "Password is not strong enough",
                    }));
                  } else {
                    setError((er) => ({ ...er, password: null }));
                  }
                }}
                id="password"
                name="password"
                type="password"
                autoComplete="current-password"
                required
                className="block w-full rounded-md border-0 py-1.5 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 focus:ring-2 focus:ring-inset focus:ring-indigo-600 sm:text-sm sm:leading-6"
              />
            </div>
          </div>
          <ReCAPTCHA
            ref={recaptchaRef}
            size="normal"
            data-theme="light"
            sitekey={process.env.NEXT_PUBLIC_RECAPTCHA_SITE_KEY!}
            onChange={onReCAPTCHAChange}
          />
          <div>
            <button
              type="submit"
              className="flex w-full justify-center rounded-md bg-indigo-600 px-3 py-1.5 text-sm font-semibold leading-6 text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600"
            >
              Sign in
            </button>
          </div>
          <div className="space-y-2">
            {error.usernameOrEmail && (
              <p className="text-sm font-semibold text-red-500">
                {error.usernameOrEmail}
              </p>
            )}
            {error.password && (
              <p className="text-sm font-semibold text-red-500">
                {error.password}
              </p>
            )}
            {error.captcha && (
              <p className="text-sm font-semibold text-red-500">
                {error.captcha}
              </p>
            )}
          </div>
        </form>

        <p className="mt-10 text-center text-sm text-gray-500">
          Not a member?{" "}
          <Link
            href="/register"
            className="font-semibold leading-6 text-indigo-600 hover:text-indigo-500"
          >
            Register
          </Link>
        </p>
      </div>
    </div>
  );
}
