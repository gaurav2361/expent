import { View, Image } from 'react-native';
import { Text } from '@/components/ui/text';
import { Button } from '@/components/ui/button';
import { SafeAreaView } from 'react-native-safe-area-context';
import { router } from 'expo-router';

export default function SplashOnboarding() {
  return (
    <SafeAreaView className="flex-1 bg-[#fff9e3]">
      <View className="flex-1 items-center justify-between py-12 px-6">
        {/* Logo/Icon */}
        <View className="items-center mt-10">
          <View className="bg-[#ea7a53] w-24 h-24 rounded-bl-[32px] rounded-tr-[32px] items-center justify-center mb-6">
            <Text className="text-white text-5xl font-bold tracking-tighter">R</Text>
          </View>
          <Text className="text-4xl font-bold text-[#081226] tracking-tight">Recurly</Text>
          <Text className="text-sm font-medium text-[#435875] tracking-widest uppercase mt-2">Smart Billing</Text>
        </View>

        {/* Content Section */}
        <View className="w-full items-center">
          <Text className="text-3xl font-bold text-[#081226] text-center mb-4">
            Manage your subscriptions like a pro
          </Text>
          <Text className="text-[#435875] text-center px-6 leading-6">
            Keep track of all your monthly bills and never miss a payment again with our smart billing platform.
          </Text>
        </View>

        {/* Action Button */}
        <View className="w-full gap-4">
          <Button 
            className="bg-[#081226] rounded-full h-16 w-full"
            onPress={() => router.replace('/(auth)/sign-in')}
          >
            <Text className="text-white font-bold text-lg">Get Started</Text>
          </Button>
          <Button 
            variant="ghost" 
            className="h-12 w-full"
            onPress={() => router.replace('/(tabs)')}
          >
            <Text className="text-[#081226] font-semibold">Try as Guest</Text>
          </Button>
        </View>
      </View>
    </SafeAreaView>
  );
}
