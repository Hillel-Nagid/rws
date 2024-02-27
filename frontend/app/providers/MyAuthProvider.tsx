"use client"
import React from 'react'
import { AuthProvider } from '../context/authContext'

type Props = {children: React.ReactNode}

export default function MyAuthProvider({children}: Props) {
  return (
    <AuthProvider>
        {children}
    </AuthProvider>
  )
}