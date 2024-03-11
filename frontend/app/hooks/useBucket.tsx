import axios from "axios";
import { s3Links } from "../utils/helpers";


export function useBucket(token: string) {
    // const token = localStorage.getItem('token');
    const axiosClient = axios.create({
        headers: {
          Authorization: `${token}`, //todo 
        },
    });
    async function fetchAllBuckets(): Promise<any[]> {
        const allBuckets: any[] = [];

        try {
            // Make requests to get each bucket
            const bucketPromises = s3Links.map(async (bucketName) => {
                try {
                    const response = await axiosClient.get(`http://localhost:2945/get_bucket/${bucketName.name}`);
                    const bucketData = response.data;
                    allBuckets.push(bucketData);
                } catch (error) {
                    console.error(`Error fetching bucket ${bucketName.name}:`, error);
                    // Handle the error gracefully
                }
            });

            // Wait for all requests to complete
            await Promise.all(bucketPromises);
        } catch (error) {
            console.error("Error fetching all buckets:", error);
            // Handle the error gracefully
        }

        return allBuckets;
    }

    return {
        axiosClient,
        fetchAllBuckets
    };
}