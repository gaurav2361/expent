import { useState } from 'react';
import { client } from '@/lib/api';

export function useAuth() {
  const [isLoading, setIsLoading] = useState(false);

  const signIn = async (email?: string, password?: string) => {
    setIsLoading(true);
    try {
      // Dummy check, replace with actual Better Auth call later
      console.log('Signing in with:', email);
      // Example call to backend
      const response = await client.post('/auth/sign-in/email', {
        email,
        password,
      });
      return response.data;
    } catch (error) {
      console.error(error);
      throw error;
    } finally {
      setIsLoading(false);
    }
  };

  const signUp = async (email?: string, password?: string, name?: string) => {
    setIsLoading(true);
    try {
      console.log('Signing up with:', email, name);
      // Example call to backend
      const response = await client.post('/auth/sign-up/email', {
        email,
        password,
        name,
      });
      return response.data;
    } catch (error) {
      console.error(error);
      throw error;
    } finally {
      setIsLoading(false);
    }
  };

  return {
    signIn,
    signUp,
    isLoading,
  };
}
