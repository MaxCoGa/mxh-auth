import pyotp
import qrcode
from PIL import Image
import pickle
import hashlib
from cryptography.fernet import Fernet
from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
import base64
import getpass
import yaml

with open('config.yml', 'r') as file:
    data = yaml.safe_load(file)

# Global
kdf = PBKDF2HMAC(
    algorithm=hashes.SHA256(),
    length=data['Length'],
    salt=bytes(data['Salt'], encoding='utf-8'),
    iterations=data['Iterations'],
)

class User:
    def __init__(self,create,username="",password=""):
        if create:
            self.newUser()
        else:
            print("NOT DONE")
            # self.username = username
            # self.password = password


    def newUser(self):
        self.setUsername()
        self.setPassword()

    def setUsername(self):
        self.username = input(("Username : "))

    def setPassword(self):
        pwdInput = str.encode(getpass.getpass("Password : "))
        self.password = base64.urlsafe_b64encode(kdf.derive(pwdInput))
        
    #   def get_encryptedkey(self):
    #     return self.key

    #   def get_decryptedkey(self):
    #     return fernet.decrypt(key).decode()

    def padString(self):
        print("Warning it should not use an user password! This is for testing ONLY!!!")
        decoded_string = self.password.decode("utf-8")
        base32_decoded_string = base64.b32encode(decoded_string.encode('ascii')).decode('utf-8')
        padString = base32_decoded_string.rstrip("=")
        return padString

    def getUri(self):
        print("Warning it should not use an user password! This is for testing ONLY!!!")
        padString = self.padString()
        uri = pyotp.totp.TOTP(padString).provisioning_uri(name=self.username, issuer_name=data['IssuerName'])
        return uri

print("mxh-auth")
authUser = User(True) 
print(authUser.username)
print(authUser.password)

def generate_hash(input_string):
  hash_object = hashlib.sha256(input_string.encode())
  hex_dig = hash_object.hexdigest()
  return hex_dig[:32]

# hash32_string = generate_hash(decoded_string)

print(authUser.getUri())

# Qr code generation step
qr = qrcode.QRCode(version=12,
                   error_correction=qrcode.constants.ERROR_CORRECT_H,
                   box_size=2,
                   border=8)

qrCodeImageName = "qrcode.png"
#qrcode.make(uri).save(qrCodeImageName)

qr.add_data(authUser.getUri())
qr.make(fit=True)
img = qr.make_image(fill_color='#f4e8ff', back_color='#120024')

img.save(qrCodeImageName)

img = Image.open(qrCodeImageName)
img.show()

totp = pyotp.TOTP(authUser.padString())
print("Current OTP:", totp.now())

# test otp from auth app
while True:
  print(totp.verify(input(("Enter the Code : "))))