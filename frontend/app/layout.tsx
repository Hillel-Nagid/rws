import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.css";
import Navbar from "./components/navbar";
import MyAuthProvider from "./providers/MyAuthProvider";

const inter = Inter({ subsets: ["latin"] });

export const metadata: Metadata = {
  title: "RWS Services",
  description: "Website description",
  icons: {
    icon: [
      {
        media: "(prefers-color-scheme: light)",
        url: "/images/favicon.png",
        href: "/images/favicon.png",
      },
      {
        media: "(prefers-color-scheme: dark)",
        url: "/images/favicon.png",
        href: "/images/favicon.png",
      },
    ],
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body className={inter.className}>
        <MyAuthProvider>
          <Navbar />
          {children}
          {/* </div> */}
        </MyAuthProvider>
      </body>
    </html>
  );
}
