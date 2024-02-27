import {
    createContext,
    ReactNode,
    useContext,
    useState,
    useEffect,
  } from "react";
  
  interface AuthContextProps {
    user: null | ILogin;
    token: null | string;
    login: (user: any, token: string) => void;
    logout: () => void;
    updateUser: (updatedUser: any) => void;
  }
  
  const AuthContext = createContext<AuthContextProps | undefined>(undefined);
  
  interface AuthProviderProps {
    children: ReactNode;
  }
  
  export function AuthProvider({ children }: AuthProviderProps) {
    const [user, setUser] = useState<null | ILogin>(null);
    const [token, setToken] = useState<null | string>(null);
  
    useEffect(() => {
      // Check local storage for existing user and token on initial render
      // const storedUser = localStorage.getItem("user");
      const storedToken = localStorage.getItem("token");
  
      if (storedToken) {
        // setUser(JSON.parse(storedUser));
        setToken(storedToken);
      }
    }, []);
  
    const login = (newUser: any, newToken: string) => {
      // setUser(newUser);
      setToken(newToken);
      // localStorage.setItem("user", JSON.stringify(newUser));
      localStorage.setItem("token", newToken);
    };
  
    const logout = () => {
      // setUser(null);
      setToken(null);
      // localStorage.removeItem("user");
      localStorage.removeItem("token");
      localStorage.removeItem("_grecaptcha")
    };
  
    const updateUser = (updatedUser: any) => {
      setUser(updatedUser);
      localStorage.setItem("user", JSON.stringify(user));
    };
  
    const contextValue: AuthContextProps = {
      user,
      token,
      login,
      logout,
      updateUser,
    };
  
    return (
      <AuthContext.Provider value={contextValue}>{children}</AuthContext.Provider>
    );
  }
  
  export const useAuth = () => {
    const context = useContext(AuthContext);
    if (!context) {
      throw new Error("useAuth must be used within an AuthProvider");
    }
    return context;
  };
  