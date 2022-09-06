#!/bin/bash
#\ & goto windows

##### unix code #####
# run this script to compile the settings schema

export PROJECT_DIRECTORY=$(pwd)
export SCHEMA_FILE=$PROJECT_DIRECTORY/assets/settings.gschema.xml
export HOME_DIRECTORY=/home/$(whoami)
export SCHEMA_COPY_LOCATION=$HOME_DIRECTORY/.local/share/glib-2.0/schemas/

mkdir -p $SCHEMA_COPY_LOCATION
cp $SCHEMA_FILE $SCHEMA_COPY_LOCATION
glib-compile-schemas $SCHEMA_COPY_LOCATION

##### windows code #####
:windows
@echo off
set PROJECT_DIRECTORY=%cd%
set SCHEMA_FILE=%PROJECT_DIRECTORY%\assets\settings.gschema.xml
set SCHEMA_COPY_LOCATION=C:\ProgramData\glib-2.0\schemas

mkdir %SCHEMA_COPY_LOCATION%
copy %SCHEMA_FILE% %SCHEMA_COPY_LOCATION%
glib-compile-schemas %SCHEMA_COPY_LOCATION%