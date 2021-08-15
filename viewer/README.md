# Viewer

デバッグ用の画像表示viewer。opengl + sdl2 + imguiで動作。

## 構成

MVPアーキテクチャをベースとした構成とした。

- App : user interface. viewerを初期化、起動する。opengl, sdl2の初期化もここで実施される。
- ImageManager : MVPにおけるModelクラス。画像(`Image`クラス)を保持し、
  `Presenter` structとデータのやり取りを行う。
  - Image : 画像についての情報を保持するstruct。
- Presenter: MVPにおけるPresenterクラス。Frame Buffer Object(FBO)を保持し、
  `ImageManager`クラスから取得したデータをFBOに描画 (off screen rendering)する
  - PresenterMode : FBOへの描画に関するcontext（現在の状態 : shader情報、画像情報など）を保持するクラス。
- Viewer : MVPにおけるViewerクラス。`Presenter`のFBOを実際のscreenに描画する。
  eventを受け取り、`Presenter`に通知する。
- Shader : GLSLのshader scriptをcompile, 保持するクラス。
- Vertex : OpenGLのvertex shaderに渡す頂点情報を保持したクラス。
