import React, {
  ReactEventHandler,
  Suspense,
  useEffect,
  useRef,
  useState,
} from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { Canvas, useLoader, useFrame } from "@react-three/fiber";
import { dialog } from "@tauri-apps/api";
import { Button } from "@mui/material";
import * as THREE from "three";
import imgUrl from "../src-tauri/icons/128x128.png";
import { MeshBasicMaterial } from "three";

type Image = {
  size: number[];
  data: number[];
};

type ImageProps = {
  selected: string[];
};

const createTexture = (): THREE.Texture => {
  //-------- ----------
  // TEXTURE
  //-------- ----------
  // USING THREE DATA TEXTURE To CREATE A RAW DATA TEXTURE
  const width = 32,
    height = 32;
  const size = width * height;
  const data = new Uint8Array(4 * size);
  for (let i = 0; i < size; i++) {
    const stride = i * 4,
      a1 = i / size,
      a2 = (i % width) / width;
    // set r, g, b, and alpha data values
    data[stride] = Math.floor(255 * a1); // red
    data[stride + 1] = 255 - Math.floor(255 * a1); // green
    data[stride + 2] = Math.floor(255 * a2); // blue
    data[stride + 3] = 255; // alpha
  }
  const texture = new THREE.DataTexture(data, width, height);
  texture.needsUpdate = true;
  return texture;
};

const loadImageTexture = async (path: string): Promise<THREE.Texture> => {
  const image = await invoke("read_image", { path });
  const texture = new THREE.DataTexture(
    new Uint8Array(image.data),
    image.size[0],
    image.size[1],
    THREE.RGBAFormat
  );
  texture.needsUpdate = true;
  return texture;
};

const test = () => {
  //-------- ----------
  // SCENE, CAMERA, RENDERER
  //-------- ----------
  const scene = new THREE.Scene();
  const camera = new THREE.PerspectiveCamera(60, 320 / 240, 0.1, 1000);
  camera.position.set(2, 2, 2);
  camera.lookAt(0, 0, 0);
  const renderer = new THREE.WebGLRenderer();
  renderer.setSize(640, 480);
  (document.getElementById("demo") || document.body).appendChild(
    renderer.domElement
  );

  const texture = createTexture();
  //-------- ----------
  // MESH
  //-------- ----------
  const plane = new THREE.Mesh(
    new THREE.PlaneGeometry(3, 3, 1, 1),
    new THREE.MeshBasicMaterial({
      map: texture,
      side: THREE.DoubleSide,
    })
  );
  scene.add(plane);
  //-------- ----------
  // RENDER
  //-------- ----------
  renderer.render(scene, camera);
};

const Image = (props: ImageProps) => {
  console.log("Render Image");
  const { selected } = props;
  const materialRef = useRef<MeshBasicMaterial>();
  const mapRef = useRef<THREE.Texture>(createTexture());
  const [cnt, setCnt] = useState(0);

  if (selected.length > 0) {
    (async () => {
      mapRef.current = await loadImageTexture(selected[0]);
      // setCnt(cnt + 1);
    })();
  }

  useFrame(() => {
    if (materialRef.current) {
      materialRef.current.map = mapRef.current;
    }
  });

  // useLoader();
  console.log("map : ", mapRef);

  return (
    <>
      <ambientLight intensity={0.1} />
      <directionalLight color="red" position={[0, 0, 5]} />
      <mesh>
        <planeBufferGeometry args={[2, 2]} />
        <meshBasicMaterial ref={materialRef} map={mapRef.current} />
      </mesh>
      <mesh position={[-3, -3, 0]}>
        <sphereGeometry args={[3, 3, 3]} />
        <meshBasicMaterial color="blue" />
      </mesh>
    </>
  );
};

const App = () => {
  console.log("Render App");
  const [filelist, setFilelist] = useState<string[]>([]);

  const onClickButton = (
    event: React.MouseEvent<HTMLButtonElement, MouseEvent>
  ) => {
    (async () => {
      const selected = await dialog.open({
        multiple: true,
        filters: [
          {
            name: "Image",
            extensions: ["jpg", "png", "gif"],
          },
        ],
      });

      if (selected === null) {
        console.log("no file selected");
        return;
      } else if (Array.isArray(selected)) {
        setFilelist(selected);
      }
    })();
  };
  useEffect(() => {
    test();
  }, []);

  console.log("filelist : ", filelist);

  return (
    <>
      <button onClick={onClickButton}>Open</button>
      <Button variant="contained" onClick={onClickButton}>
        Open by mui
      </Button>
      <Canvas>
        <Suspense fallback={null}>
          <Image selected={filelist} />
        </Suspense>
      </Canvas>
    </>
  );
};

export default App;

// function Scene() {
//   // const colorMap = useLoader(TextureLoader, "PavingStones092_1K_Color.jpg");
//   return (
//     <>
//       <ambientLight intensity={0.2} />
//       <directionalLight />
//       <mesh>
//         <sphereGeometry args={[1, 32, 32]} />
//         <meshStandardMaterial />
//       </mesh>
//     </>
//   );
// }

// export default function App() {
//   return (
//   );
// }
