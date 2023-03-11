import { useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { Canvas } from "@react-three/fiber";
import "./App.css";

const App = () => {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    setGreetMsg(await invoke("greet", { name }));
  }

  return (
    <div className="container">
      <Canvas>
        <ambientLight intensity={0.1} />
        <directionalLight color="red" position={[0, 0, 5]} />
        <mesh position={[2, 2, 0]}>
          <boxGeometry args={[2, 2, 2]} />
          <meshBasicMaterial />
        </mesh>
        <mesh position={[-2, -2, 0]}>
          <sphereGeometry args={[1, 16, 16]} />
          <meshBasicMaterial color="blue" />
        </mesh>
      </Canvas>
    </div>
  );
};

export default App;
