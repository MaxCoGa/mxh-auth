# mxh-auth

python -m venv mxh-auth-venv


source mxh-auth-venv/bin/activate

win:
Set-ExecutionPolicy Unrestricted -Scope Process
.\mxh-auth-venv\Scripts\Activate.ps1
Set-ExecutionPolicy Default -Scope Process

add module: pip freeze > requirements.txt 

pip install -r requirements.txt

python auth.py 

deactivate