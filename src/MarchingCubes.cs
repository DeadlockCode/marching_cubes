using System.Collections;
using System.Collections.Generic;
using UnityEngine;
using UnityEditor;

public class MarchingCubes : MonoBehaviour
{
    public int meshResolution = 16;

    public float meshFloor = 1;

    public float noiseScale = 1;

    public bool randomNoiseOffset;

    public Vector3 noiseOffset;

    public float noisePower = 0.1f;

    public int noiseOctaves = 1;

    public float octaveScaleMult = 2.0f;

    public float octavePowerMult = 0.5f;

    public Material material;

    private Mesh mesh;

    private new MeshCollider collider;

    private float[,,] space;

    private void Start()
    {
        space = new float[meshResolution + 1, meshResolution + 1, meshResolution + 1];

        MeshFilter filter = gameObject.AddComponent<MeshFilter>();
        mesh = new Mesh();
        filter.mesh = mesh;

        collider = gameObject.AddComponent<MeshCollider>();
        collider.sharedMesh = mesh;

        MeshRenderer renderer = gameObject.AddComponent<MeshRenderer>();
        renderer.material = material;

        Generate();
    }

    public void Hit(Vector3 position, float radius, float damage)
    {
        int pointAmount = 0;
        for (int x = 0; x < meshResolution + 1; x++)
            for (int y = 0; y < meshResolution + 1; y++)
                for (int z = 0; z < meshResolution + 1; z++)
                {
                    if ((transform.TransformPoint(new Vector3(x, y, z) / meshResolution - new Vector3(0.5f, 0.5f, 0.5f)) - position).magnitude < radius)
                        space[x, y, z] += damage;
                    if (space[x, y, z] < meshFloor)
                        pointAmount++;
                }
        if (pointAmount < Mathf.Pow(meshResolution, 3) * 0.05f)
            Destroy(gameObject);
        GenerateMesh();
    }

    public void Generate()
    {
        if (randomNoiseOffset)
        {
            noiseOffset = new Vector3(Random.Range(-10000.0f, 10000.0f), Random.Range(-10000.0f, 10000.0f), Random.Range(-10000.0f, 10000.0f));
        }

        space = new float[meshResolution + 1, meshResolution + 1, meshResolution + 1];

        GeneratePoints();
        GenerateMesh();
    }

    private void GeneratePoints()
    {
        for (int x = 0; x < meshResolution + 1; x++)
            for (int y = 0; y < meshResolution + 1; y++)
                for (int z = 0; z < meshResolution + 1; z++)
                {
                    Vector3 v = (Vector3.one * 0.5f) - (new Vector3(x, y, z) / meshResolution);
                    Vector3 n = v.normalized;

                    Vector3 coord = n * noiseScale + noiseOffset;

                    space[x, y, z] = v.magnitude;

                    for (int i = 0; i < noiseOctaves; i++)
                    {
                        float inc = Mathf.Pow(octaveScaleMult, i);
                        float dec = Mathf.Pow(octavePowerMult, i);

                        space[x, y, z] += Perlin.Noise(coord.x * inc, coord.y * inc, coord.z * inc) * dec * noisePower;
                    }
                }
    }

    private void GenerateMesh()
    {
        List<Vector3> vertices = new();
        List<int> triangles = new();

        int index = 0;
        for (int x = 0; x < meshResolution; x++)
            for (int y = 0; y < meshResolution; y++)
                for (int z = 0; z < meshResolution; z++)
                {
                    int triangulationIndex = GetTrianglulationIndex(x, y, z);
                    int[] triangulation = MarchTables.triangulation[triangulationIndex];

                    foreach (int edgeIndex in triangulation)
                    {
                        if (edgeIndex == -1) break;

                        Vector3 posA = new Vector3(x, y, z) + MarchTables.pointOffsets[MarchTables.cornerIndexAFromEdge[edgeIndex]];
                        Vector3 posB = new Vector3(x, y, z) + MarchTables.pointOffsets[MarchTables.cornerIndexBFromEdge[edgeIndex]];

                        float valA = space[(int)posA.x, (int)posA.y, (int)posA.z];
                        float valB = space[(int)posB.x, (int)posB.y, (int)posB.z];

                        float t = (meshFloor - valB) / (valA - valB);


                        Vector3 vertexPos = posB + (posA - posB) * t;

                        bool makeVertex = true;
                        for (int i = 0; i < vertices.Count; i++)
                        {
                            if (vertices[i] == vertexPos / meshResolution - Vector3.one * 0.5f)
                            {
                                triangles.Add(i);
                                makeVertex = false;
                            }
                        }
                        
                        if (makeVertex)
                        {
                            vertices.Add(vertexPos / meshResolution - Vector3.one * 0.5f);
                        
                            triangles.Add(index);
                            index++;
                        }
                    }
                }

        mesh.Clear();

        mesh.vertices = vertices.ToArray();
        mesh.triangles = triangles.ToArray();

        mesh.RecalculateNormals();

        collider.sharedMesh = mesh;
    }

    private int GetTrianglulationIndex(int x, int y, int z)
    {
        int triangulationIndex = 0;
        if (space[  x  ,  y  ,  z  ] > meshFloor) triangulationIndex |= 1 << 0;
        if (space[  x  ,  y  ,z + 1] > meshFloor) triangulationIndex |= 1 << 1;
        if (space[x + 1,  y  ,z + 1] > meshFloor) triangulationIndex |= 1 << 2;
        if (space[x + 1,  y  ,  z  ] > meshFloor) triangulationIndex |= 1 << 3;
        if (space[  x  ,y + 1,  z  ] > meshFloor) triangulationIndex |= 1 << 4;
        if (space[  x  ,y + 1,z + 1] > meshFloor) triangulationIndex |= 1 << 5;
        if (space[x + 1,y + 1,z + 1] > meshFloor) triangulationIndex |= 1 << 6;
        if (space[x + 1,y + 1,  z  ] > meshFloor) triangulationIndex |= 1 << 7;
        return triangulationIndex;
    }
}

[CustomEditor(typeof(MarchingCubes))]
public class MarchingCubesEditor : Editor
{
    public override void OnInspectorGUI()
    {
        DrawDefaultInspector();
        MarchingCubes marchingCubes = (MarchingCubes)target;

        if(GUILayout.Button("Generate"))
            marchingCubes.Generate();
    }
}