import { ForgotPasswordForm } from '@/components/forgot-password-form';
import { SafeAreaView } from 'react-native-safe-area-context';
import { KeyboardAvoidingView, Platform, ScrollView, View } from 'react-native';

export default function ForgotPasswordScreen() {
  return (
    <SafeAreaView className="flex-1 bg-[#fff9e3]">
      <KeyboardAvoidingView
        behavior={Platform.OS === 'ios' ? 'padding' : 'height'}
        className="flex-1"
      >
        <ScrollView contentContainerStyle={{ flexGrow: 1 }} keyboardShouldPersistTaps="handled">
          <View className="flex-1 px-6 justify-center">
            <ForgotPasswordForm />
          </View>
        </ScrollView>
      </KeyboardAvoidingView>
    </SafeAreaView>
  );
}
