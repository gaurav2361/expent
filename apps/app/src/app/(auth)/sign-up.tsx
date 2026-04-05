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

export default function SignUpScreen() {
  const [name, setName] = useState('');
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const { signUp, isLoading } = useAuth();

  const handleSignUp = async () => {
    if (!name || !email || !password || !confirmPassword) {
      showErrorMessage('Please fill in all fields');
      return;
    }

    if (password !== confirmPassword) {
      showErrorMessage('Passwords do not match');
      return;
    }

    try {
      await signUp(email, password, name);
      router.replace('/(tabs)');
    } catch (error) {
      showErrorMessage(error instanceof Error ? error.message : 'Failed to sign up');
    }
  };

  return (
    <SafeAreaView className="flex-1 bg-[#fff9e3]">
      <KeyboardAvoidingView 
        behavior={Platform.OS === 'ios' ? 'padding' : 'height'}
        className="flex-1"
      >
        <ScrollView contentContainerStyle={{ flexGrow: 1 }} keyboardShouldPersistTaps="handled">
          <View className="flex-1 px-6 pt-8 justify-center">
            {/* Logo Section */}
            <View className="flex-row items-center justify-center gap-3 mb-10 mt-6">
              <View className="bg-[#ea7a53] w-12 h-12 rounded-bl-[16px] rounded-tr-[16px] items-center justify-center">
                <Text className="text-white text-3xl font-bold tracking-tighter">R</Text>
              </View>
              <Text className="text-2xl font-bold text-[#081226] tracking-tight">Recurly</Text>
            </View>

            {/* Welcome Text */}
            <View className="items-center mb-8">
              <Text className="text-2xl font-bold text-[#081226] mb-2">Create an account</Text>
              <Text className="text-[#435875] text-center">Join us to start managing your billing smartly</Text>
            </View>

            {/* Form Container */}
            <View className="bg-white/80 border border-[#e1dbca] rounded-[32px] p-6 shadow-sm shadow-black/5 gap-5">
              <View className="gap-2">
                <Label nativeID="name-label" className="text-[#081226] font-semibold ml-1">Full Name</Label>
                <Input 
                  placeholder="Enter your name" 
                  aria-labelledby="name-label"
                  className="bg-[#fff9e3] border-[#c6bfa2]/40 h-14 rounded-2xl px-4 text-[#081226]"
                  value={name}
                  onChangeText={setName}
                  autoCapitalize="words"
                />
              </View>

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
                  placeholder="Create a password" 
                  secureTextEntry
                  aria-labelledby="password-label"
                  className="bg-[#fff9e3] border-[#c6bfa2]/40 h-14 rounded-2xl px-4 text-[#081226]"
                  value={password}
                  onChangeText={setPassword}
                />
              </View>

              <View className="gap-2">
                <Label nativeID="confirm-password-label" className="text-[#081226] font-semibold ml-1">Confirm Password</Label>
                <Input 
                  placeholder="Confirm your password" 
                  secureTextEntry
                  aria-labelledby="confirm-password-label"
                  className="bg-[#fff9e3] border-[#c6bfa2]/40 h-14 rounded-2xl px-4 text-[#081226]"
                  value={confirmPassword}
                  onChangeText={setConfirmPassword}
                />
              </View>

              <Button 
                className="bg-[#ea7a53] rounded-2xl py-4 h-14 mt-4 shadow-sm shadow-[#ea7a53]/20"
                onPress={handleSignUp}
                disabled={isLoading}
              >
                <Text className="text-white font-bold text-lg">{isLoading ? 'Creating account...' : 'Create account'}</Text>
              </Button>

              <View className="flex-row justify-center gap-1 mt-2">
                <Text className="text-[#435875]">Already have an account?</Text>
                <Link href="/(auth)/sign-in" asChild>
                  <Text className="text-[#ea7a53] font-bold">Sign in</Text>
                </Link>
              </View>
            </View>
          </View>
        </ScrollView>
      </KeyboardAvoidingView>
    </SafeAreaView>
  );
}
