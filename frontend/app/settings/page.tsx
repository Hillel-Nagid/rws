'use client'
import { useRouter } from "next/navigation";
import { useEffect } from "react";

export default function SignInPage() {
  const router = useRouter();
  // Access token from AuthContext
  let token; 
 useEffect(() => {
   // Check if localStorage is available
   if (typeof window !== "undefined") {
     token = localStorage.getItem("token");
     if(!token) {
       router.push('/signin');
     }
   }
 }, [token, router]);
  return (
    <div>
      <h1>settings</h1>
    </div>
  )
}
