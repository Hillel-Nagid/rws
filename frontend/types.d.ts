interface IRegister {
  email: string;
  username: string;
  password: string;
}
interface IRegisterError {
  email: string | null;
  username: string | null;
  password: string | null;
}

interface ILogin {
  usernameOrEmail: string;
  password: string;
}

interface ILoginError {
  usernameOrEmail: string | null;
  password: string | null;
  captcha: string | null;
}

