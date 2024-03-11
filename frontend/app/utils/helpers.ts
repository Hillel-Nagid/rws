import { useEffect } from "react";
import { GrBucket } from "react-icons/gr";

export const navigation = [
    { name: 'Welcome Page', href: '/homepage'},
    { name: 'Home', href: '/homepage'}, // check for token every time you are using it
    { name: 'Settings', href: '/settings'},// check for token every time you are using it
    { name: 'About us', href: '/about'},
] as const

//TODO: contains all the product=services we have. when there is a new service, you have to add it here!!
export const s3Links=[
  { name: 'S3', href: '/homepage/s3',Icon:GrBucket },
  // { name: 'S4', href: '/homepage/s4',Icon:GrBucket },
  // { name: 'S5', href: '/homepage/s3',Icon:GrBucket },
] as const 

export interface Product {
  name: string;
  href: string;
  disabled: boolean;
}