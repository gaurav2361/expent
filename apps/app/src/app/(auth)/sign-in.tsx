import { View } from 'react-native';
import { Text } from '@/components/ui/text';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { SafeAreaView } from 'react-native-safe-area-context';
import { Link, router } from 'expo-router';

export default function SignInScreen() {
  return (
    <SafeAreaView className="flex-1 bg-[#fff9e3]">
      <View className="flex-1 px-4 pt-12">
        {/* Logo Section */}
        <View className="flex-row items-center justify-center gap-3 mb-20">
          <View className="bg-[#ea7a53] w-16 h-16 rounded-bl-[20px] rounded-tr-[20px] items-center justify-center">
            <Text className="text-white text-4xl font-bold tracking-tighter">R</Text>
          </View>
          <View>
            <Text className="text-2xl font-bold text-[#081226] tracking-tight">Recurly</Text>
            <Text className="text-[10px] font-medium text-[#435875] tracking-widest uppercase">Smart Billing</Text>
          </View>
        </View>

        {/* Welcome Text */}
        <View className="items-center mb-10">
          <Text className="text-2xl font-bold text-[#081226] mb-2">Welcome back</Text>
          <Text className="text-[#435875] text-center px-8">Sign in to continue managing your subscriptions</Text>
        </View>

        {/* Form Container */}
        <View className="bg-white/50 border border-[#e1dbca] rounded-[24px] p-6 gap-6">
          <View className="gap-2">
            <Label nativeID="email-label" className="text-[#081226] font-semibold">Email</Label>
            <Input 
              placeholder="Enter your email" 
              aria-labelledby="email-label"
              className="bg-[#fff9e3] border-[#c6bfa2] h-14 rounded-[14px] px-4"
            />
          </View>

          <View className="gap-2">
            <Label nativeID="password-label" className="text-[#081226] font-semibold">Password</Label>
            <Input 
              placeholder="Enter your password" 
              secureTextEntry
              aria-labelledby="password-label"
              className="bg-[#fff9e3] border-[#c6bfa2] h-14 rounded-[14px] px-4"
            />
          </View>

          <Button 
            className="bg-[#ea7a53] rounded-[14px] py-4 h-14 mt-2"
            onPress={() => router.replace('/(tabs)')}
          >
            <Text className="text-white font-bold text-lg">Sign in</Text>
          </Button>

          <View className="flex-row justify-center gap-1">
            <Text className="text-[#435875]">New to Recurly?</Text>
            <Link href="/(auth)/sign-up" asChild>
              <Text className="text-[#ea7a53] font-semibold">Create an account</Text>
            </Link>
          </View>
        </View>
      </View>
    </SafeAreaView>
  );
}
