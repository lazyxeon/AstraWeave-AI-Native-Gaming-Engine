@echo off
setlocal ENABLEDELAYEDEXPANSION
echo == AstraWeave Graphics Check ==

call cargo check -p astraweave-render || goto :error
call cargo check -p visual_3d || goto :error
call cargo check -p weaving_playground || goto :error
call cargo check -p physics_demo3d || goto :error
call cargo check -p terrain_demo || goto :error
call cargo check -p cutscene_render_demo || goto :error
call cargo check -p unified_showcase || goto :error

echo All graphics checks completed.
goto :eof

:error
echo Graphics check failed with error code %ERRORLEVEL%.
exit /b %ERRORLEVEL%
