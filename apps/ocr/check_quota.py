import os
import sys
from dotenv import load_dotenv
from google import genai


def check_gemini_quota():
    # Try multiple .env locations
    for env_path in [".env", "apps/ocr/.env", "../../.env"]:
        load_dotenv(env_path, override=True)

    api_key = os.getenv("GOOGLE_API_KEY")

    if not api_key:
        print("❌ Error: GOOGLE_API_KEY not found in environment or .env file.")
        return False

    client = genai.Client(api_key=api_key)
    model_id = os.getenv("GEMINI_MODEL", "gemini-2.5-flash")

    print(f"Checking quota for model: {model_id}...")

    try:
        # Minimal request to check connectivity and quota
        response = client.models.generate_content(model=model_id, contents="Say 'OK' if you can hear me.")

        print(f"✅ Quota OK! Gemini responded: {response.text.strip()}")
        return True

    except Exception as e:
        error_msg = str(e)
        if "429" in error_msg or "quota" in error_msg.lower():
            print("\n❌ QUOTA EXCEEDED (429 Too Many Requests)")
            print("Message:", error_msg)
            # Try to extract retry time if available
            if "retry in" in error_msg.lower():
                import re

                match = re.search(r"retry in ([\d\.]+)s", error_msg.lower())
                if match:
                    print(f"💡 You can retry in approximately {match.group(1)} seconds.")
        elif "403" in error_msg or "permission" in error_msg.lower():
            print("\n❌ PERMISSION DENIED (403)")
            print("Your API key might be invalid or doesn't have access to this model.")
        else:
            print(f"\n❌ UNKNOWN ERROR: {error_msg}")

        return False


if __name__ == "__main__":
    success = check_gemini_quota()
    sys.exit(0 if success else 1)
