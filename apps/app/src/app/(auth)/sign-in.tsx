import { View, KeyboardAvoidingView, Platform, ScrollView } from 'react-native';
import { Text } from '@/components/ui/text';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { SafeAreaView } from 'react-native-safe-area-context';
import { Link, router } from 'expo-router';
import { useState } from 'react';
import { useAuth } from '@/lib/auth/use-auth';
import { showErrorMessage } from '@/components/ui/utils';

export default function SignInScreen() {
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const { signIn, isLoading } = useAuth();

  const handleSignIn = async () => {
    if (!email || !password) {
      showErrorMessage('Please enter both email and password');
      return;
    }

    try {
      await signIn(email, password);
      router.replace('/(tabs)');
    } catch (error) {
      showErrorMessage(error instanceof Error ? error.message : 'Failed to sign in');
    }
  };

  return (
    <SafeAreaView className="flex-1 bg-[#fff9e3]">
      <KeyboardAvoidingView 
        behavior={Platform.OS === 'ios' ? 'padding' : 'height'}
        className="flex-1"
      >
        <ScrollView contentContainerStyle={{ flexGrow: 1 }} keyboardShouldPersistTaps="handled">
          <View className="flex-1 px-6 pt-12 justify-center">
            {/* Logo Section */}
            <View className="flex-row items-center justify-center gap-3 mb-12 mt-8">
              <View className="bg-[#ea7a53] w-16 h-16 rounded-bl-[20px] rounded-tr-[20px] items-center justify-center">
                <Text className="text-white text-4xl font-bold tracking-tighter">R</Text>
              </View>
              <View>
                <Text className="text-3xl font-bold text-[#081226] tracking-tight">Recurly</Text>
                <Text className="text-[10px] font-bold text-[#435875] tracking-widest uppercase">Smart Billing</Text>
              </View>
            </View>

            {/* Welcome Text */}
            <View className="items-center mb-8">
              <Text className="text-2xl font-bold text-[#081226] mb-2">Welcome back</Text>
              <Text className="text-[#435875] text-center">Sign in to continue managing your subscriptions</Text>
            </View>

            {/* Form Container */}
            <View className="bg-white/80 border border-[#e1dbca] rounded-[32px] p-6 shadow-sm shadow-black/5 gap-5">
              <View className="gap-2">
                <Label nativeID="email-label" className="text-[#081226] font-semibold ml-1">Email</Label>
                <Input 
                  placeholder="Enter your email" 
                  aria-labelledby="email-label"
                  className="bg-[#fff9e3] border-[#c6bfa2]/40 h-14 rounded-2xl px-4 text-[#081226]"
                  value={email}
                  onChangeText={setEmail}
                  autoCapitalize="none"
                  keyboardType="email-address"
                />
              </View>

              <View className="gap-2">
                <Label nativeID="password-label" className="text-[#081226] font-semibold ml-1">Password</Label>
                <Input 
                  placeholder="Enter your password" 
                  secureTextEntry
                  aria-labelledby="password-label"
                  className="bg-[#fff9e3] border-[#c6bfa2]/40 h-14 rounded-2xl px-4 text-[#081226]"
                  value={password}
                  onChangeText={setPassword}
                />
              </View>

              <Button 
                className="bg-[#ea7a53] rounded-2xl py-4 h-14 mt-4 shadow-sm shadow-[#ea7a53]/20"
                onPress={handleSignIn}
                disabled={isLoading}
              >
                <Text className="text-white font-bold text-lg">{isLoading ? 'Signing in...' : 'Sign in'}</Text>
              </Button>

              <View className="flex-row justify-center gap-1 mt-2">
                <Text className="text-[#435875]">New to Recurly?</Text>
                <Link href="/(auth)/sign-up" asChild>
                  <Text className="text-[#ea7a53] font-bold">Create an account</Text>
                </Link>
              </View>
            </View>
          </View>
        </ScrollView>
      </KeyboardAvoidingView>
    </SafeAreaView>
  );
}
