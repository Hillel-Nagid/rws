'use client';
import React, { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import { useBucket } from "../hooks/useBucket";
import { Product } from "../utils/helpers";

export default function Page() {
  const router = useRouter();
  const [products, setProducts] = useState<Product[]>([]);
  const [token, setToken] = useState<string | null>(null);

  useEffect(() => {
    // Check for the token during initial render
    if (typeof window !== "undefined") {
      const storedToken = localStorage.getItem("token");
      if (!storedToken) {
        router.push("/signin");
      } else {
        setToken(storedToken);
      }
    }
  }, [router]);

  useEffect(() => {
    // Fetch data only when token is available
    if (token) {
      fetchData();
    }
  }, [token]);

  async function fetchData() {
    try {
      if(!token) return;
      const { fetchAllBuckets } = useBucket(token);
      const buckets = await fetchAllBuckets();
      const productsWithPlaceholders: Product[] = [
        ...buckets,
        ...Array.from({ length: Math.max(6 - buckets.length, 0) }, (_, index) => ({
          name: "Coming Soon",
          href: `#`,
          disabled: true,
        })),
      ];
      setProducts(productsWithPlaceholders);
    } catch (error) {
      console.error("Error fetching buckets:", error);
    }
  }

  return (
    <>
      <div>Home Page</div>
      <div>
        <h3>Popular</h3>
        <div>
          {products.map((product, index) => (
            <React.Fragment key={index}>
              {product.disabled ? (
                <span style={{ color: "gray", textDecoration: "line-through" }}>
                  {product.name}
                </span>
              ) : (
                <Link href={product.href}>{product.name}</Link>
              )}
            </React.Fragment>
          ))}
        </div>
      </div>
    </>
  );
}
